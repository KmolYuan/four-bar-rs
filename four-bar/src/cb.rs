//! Create a codebook database for four-bar linkages.
use super::{
    planar_syn::{Mode, MIN_ANGLE},
    FourBar, NormFourBar,
};
use efd::{Efd2, Transform2};
use mh::{
    random::{Rng, SeedOption},
    utility::prelude::*,
};
use std::{
    io::{Read, Seek, Write},
    sync::Mutex,
};

mod distr;

fn to_arr<A, D>(stack: Mutex<Vec<Array<A, D>>>, n: usize) -> Array<A, D::Larger>
where
    A: Clone,
    D: Dimension,
{
    let stack = stack.into_inner().unwrap();
    let arrays = stack.iter().take(n).map(Array::view).collect::<Vec<_>>();
    ndarray::stack(Axis(0), &arrays).unwrap()
}

/// Codebook generation config.
pub struct Cfg {
    /// Open curve
    pub is_open: bool,
    /// Number of data
    pub size: usize,
    /// Number of curve coordinates
    pub res: usize,
    /// Harmonic
    pub harmonic: usize,
    /// Random seed
    pub seed: SeedOption,
}

impl Default for Cfg {
    fn default() -> Self {
        Self::new()
    }
}

impl Cfg {
    /// Constant default value.
    pub const fn new() -> Self {
        Self {
            is_open: false,
            size: 102400,
            res: 720,
            harmonic: 20,
            seed: SeedOption::None,
        }
    }

    mh::impl_builders! {
        /// Open curve
        fn is_open(bool)
        /// Number of data
        fn size(usize)
        /// Number of curve coordinates
        fn res(usize)
        /// Harmonic
        fn harmonic(usize)
        /// Random seed
        fn seed(SeedOption)
    }
}

/// Codebook type.
#[derive(Clone)]
pub struct Codebook {
    fb: Array2<f64>,
    inv: Array1<bool>,
    efd: Array3<f64>,
}

impl Default for Codebook {
    fn default() -> Self {
        Self {
            fb: Array2::default([0, 5]),
            inv: Array1::default(0),
            efd: Array3::default([0, 0, 4]),
        }
    }
}

impl Codebook {
    /// Takes time to generate codebook data.
    pub fn make(cfg: Cfg) -> Self {
        Self::make_with(cfg, |_| ())
    }

    /// Takes time to generate codebook data with a callback function.
    pub fn make_with<C>(cfg: Cfg, callback: C) -> Self
    where
        C: Fn(usize) + Sync + Send,
    {
        let Cfg { is_open, size, res, harmonic, seed } = cfg;
        let rng = Rng::new(seed);
        let fb_stack = Mutex::new(Vec::with_capacity(size));
        let inv_stack = Mutex::new(Vec::with_capacity(size));
        let efd_stack = Mutex::new(Vec::with_capacity(size));
        loop {
            let len = efd_stack.lock().unwrap().len();
            let n = (size - len) / 2;
            #[cfg(feature = "rayon")]
            let iter = rng.stream(n).into_par_iter();
            #[cfg(not(feature = "rayon"))]
            let iter = rng.stream(n).into_iter();
            iter.flat_map(|rng| rng.sample(self::distr::NormFbDistr))
                .filter(|fb| is_open == fb.ty().is_open_curve())
                .filter_map(|fb| {
                    let [t1, t2] = fb.angle_bound().filter(|[t1, t2]| t2 - t1 > MIN_ANGLE)?;
                    Some((fb.curve_in(t1, t2, res), fb))
                })
                .filter(|(c, _)| c.len() > 1)
                .for_each(|(curve, fb)| {
                    let mode = if is_open { Mode::Open } else { Mode::Closed };
                    let curve = mode.regularize(curve);
                    let efd = Efd2::from_curve_harmonic(curve, harmonic).unwrap();
                    efd_stack.lock().unwrap().push(efd.into_inner());
                    let mut stack = fb_stack.lock().unwrap();
                    stack.push(arr1(&fb.as_array()));
                    callback(stack.len());
                    inv_stack.lock().unwrap().push(arr0(fb.inv()));
                });
            if efd_stack.lock().unwrap().len() >= size {
                break;
            }
        }
        let fb = to_arr(fb_stack, size);
        let inv = to_arr(inv_stack, size);
        let efd = to_arr(efd_stack, size);
        Self { fb, inv, efd }
    }

    /// Read codebook from NPZ file.
    pub fn read(r: impl Read + Seek) -> Result<Self, ndarray_npy::ReadNpzError> {
        let mut r = ndarray_npy::NpzReader::new(r)?;
        macro_rules! impl_read {
            ($r:ident, $($field:ident),+) => {{
                $(let $field = $r.by_name(stringify!($field))?;)+
                Ok(Self { $($field),+ })
            }};
        }
        impl_read!(r, fb, inv, efd)
    }

    /// Write codebook to NPZ file.
    pub fn write<W>(&self, w: W) -> Result<W, ndarray_npy::WriteNpzError>
    where
        W: Write + Seek,
    {
        let mut w = ndarray_npy::NpzWriter::new_compressed(w);
        macro_rules! impl_write {
            ($w:ident, $($field:ident),+) => {
                let Self { $($field),+ } = self;
                $($w.add_array(stringify!($field), $field)?;)+
            };
        }
        impl_write!(w, fb, inv, efd);
        w.finish()
    }

    /// Total size.
    pub fn size(&self) -> usize {
        self.fb.nrows()
    }

    /// Clear the codebook.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Return whether the codebook has any data.
    pub fn is_empty(&self) -> bool {
        self.fb.is_empty()
    }

    /// Number of the harmonics.
    pub fn harmonic(&self) -> usize {
        self.efd.len_of(Axis(1))
    }

    /// Get the n-nearest four-bar linkages from a target curve.
    ///
    /// This method will keep the dimensional variables without transform.
    pub fn fetch_raw(&self, target: &[[f64; 2]], size: usize) -> Vec<(f64, NormFourBar)> {
        if self.is_empty() {
            return Vec::new();
        }
        let target = Efd2::from_curve_harmonic(target, self.harmonic()).unwrap();
        #[cfg(feature = "rayon")]
        let iter = self.efd.axis_iter(Axis(0)).into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = self.efd.axis_iter(Axis(0));
        let dis = iter
            .map(|efd| target.l1_norm(&Efd2::try_from_coeffs(efd.to_owned()).unwrap()))
            .collect::<Vec<_>>();
        if size == 1 {
            return dis
                .into_iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, err)| (err, self.pick_norm(i)))
                .into_iter()
                .collect();
        }
        let mut ind = (0..dis.len()).collect::<Vec<_>>();
        ind.sort_by(|&a, &b| dis[a].partial_cmp(&dis[b]).unwrap());
        ind.into_iter()
            .take(size)
            .map(|i| (dis[i], self.pick_norm(i)))
            .collect()
    }

    /// Get the nearest four-bar linkage from a target curve.
    pub fn fetch_1st(&self, target: &[[f64; 2]], res: usize) -> Option<(f64, FourBar)> {
        if self.is_empty() {
            return None;
        }
        let target = Efd2::from_curve_harmonic(target, self.harmonic()).unwrap();
        #[cfg(feature = "rayon")]
        let iter = self.efd.axis_iter(Axis(0)).into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = self.efd.axis_iter(Axis(0));
        iter.map(|efd| target.l1_norm(&Efd2::try_from_coeffs(efd.to_owned()).unwrap()))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, err)| (err, self.pick(i, target.as_trans(), res)))
    }

    /// Get the n-nearest four-bar linkages from a target curve.
    ///
    /// Slower than [`Self::fetch_1st()`].
    pub fn fetch(&self, target: &[[f64; 2]], size: usize, res: usize) -> Vec<(f64, FourBar)> {
        if self.is_empty() {
            return Vec::new();
        } else if size == 1 {
            return self.fetch_1st(target, res).into_iter().collect();
        }
        let target = Efd2::from_curve_harmonic(target, self.harmonic()).unwrap();
        #[cfg(feature = "rayon")]
        let iter = self.efd.axis_iter(Axis(0)).into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = self.efd.axis_iter(Axis(0));
        let dis = iter
            .map(|efd| target.l1_norm(&Efd2::try_from_coeffs(efd.to_owned()).unwrap()))
            .collect::<Vec<_>>();
        let mut ind = (0..self.size()).collect::<Vec<_>>();
        ind.sort_by(|&a, &b| dis[a].partial_cmp(&dis[b]).unwrap());
        ind.into_iter()
            .take(size)
            .map(|i| (dis[i], self.pick(i, target.as_trans(), res)))
            .collect()
    }

    fn pick_norm(&self, i: usize) -> NormFourBar {
        NormFourBar::try_from(self.fb.slice(s![i, ..]).as_slice().unwrap())
            .unwrap()
            .with_inv(self.inv[i])
    }

    fn pick(&self, i: usize, trans: &Transform2, res: usize) -> FourBar {
        let fb = NormFourBar::try_from(self.fb.slice(s![i, ..]).as_slice().unwrap())
            .unwrap()
            .with_inv(self.inv[i]);
        let curve = fb.curve(res);
        let efd = Efd2::from_curve(curve).unwrap();
        FourBar::from_norm(fb).transform(&efd.as_trans().to(trans))
    }

    /// Merge two data to one codebook.
    pub fn merge(&self, rhs: &Self) -> Result<Self, ndarray::ShapeError> {
        let mut cb = self.clone();
        cb.merge_inplace(rhs)?;
        Ok(cb)
    }

    /// Merge two data to one codebook inplace.
    pub fn merge_inplace(&mut self, rhs: &Self) -> Result<(), ndarray::ShapeError> {
        if self.is_empty() {
            self.clone_from(rhs);
        } else {
            macro_rules! merge {
                ($($field:ident),+) => {$(
                    self.$field = ndarray::concatenate(Axis(0), &[self.$field.view(), rhs.$field.view()])?;
                )+};
            }
            merge!(fb, inv, efd);
        }
        Ok(())
    }
}

impl FromIterator<Codebook> for Codebook {
    fn from_iter<T: IntoIterator<Item = Codebook>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|a, b| a.merge(&b).unwrap_or(a))
            .unwrap_or_default()
    }
}
