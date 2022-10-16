//! Create a codebook database for four-bar linkages.
use super::{
    syn::{Mode, BOUND},
    FourBar, NormFourBar,
};
use efd::{Efd2, Transform2};
use mh::utility::prelude::*;
use std::{
    io::{Read, Seek, Write},
    sync::Mutex,
};

fn stack<A, D>(stack: Mutex<Vec<Array<A, D>>>, n: usize) -> Array<A, D::Larger>
where
    A: Clone,
    D: Dimension,
{
    let stack = stack.into_inner().unwrap();
    let arrays = stack.iter().take(n).map(Array::view).collect::<Vec<_>>();
    ndarray::stack(Axis(0), &arrays).unwrap()
}

/// Codebook type.
#[derive(Clone)]
pub struct Codebook {
    fb: Array2<f64>,
    inv: Array1<bool>,
    efd: Array3<f64>,
    trans: Array2<f64>,
}

impl Default for Codebook {
    fn default() -> Self {
        Self {
            fb: Array2::default([0, 5]),
            inv: Array1::default(0),
            efd: Array3::default([0, 0, 4]),
            trans: Array2::default([0, 4]),
        }
    }
}

impl Codebook {
    /// Takes time to generate codebook data.
    pub fn make(is_open: bool, n: usize, res: usize, harmonic: usize) -> Self {
        Self::make_with(is_open, n, res, harmonic, |_| ())
    }

    /// Takes time to generate codebook data with a callback function.
    pub fn make_with<C>(is_open: bool, n: usize, res: usize, harmonic: usize, callback: C) -> Self
    where
        C: Fn(usize) + Sync + Send,
    {
        let rng = Rng::new(None);
        let fb_stack = Mutex::new(Vec::with_capacity(n));
        let inv_stack = Mutex::new(Vec::with_capacity(n));
        let efd_stack = Mutex::new(Vec::with_capacity(n));
        let trans_stack = Mutex::new(Vec::with_capacity(n));
        loop {
            let len = efd_stack.lock().unwrap().len();
            #[cfg(feature = "rayon")]
            let iter = (0..(n - len) / 2).into_par_iter();
            #[cfg(not(feature = "rayon"))]
            let iter = 0..(n - len) / 2;
            iter.flat_map(|_| {
                let v = BOUND[..5]
                    .iter()
                    .map(|&[u, l]| rng.float(u..l))
                    .collect::<Vec<_>>();
                [false, true].map(|inv| NormFourBar::try_from_slice(&v, inv).unwrap())
            })
            .map(|fb| match is_open {
                false => fb.to_close_curve(),
                true => fb.to_open_curve(),
            })
            .filter(|fb| is_open == fb.ty().is_open_curve())
            .filter_map(|fb| fb.curve(res).map(|c| (c, fb)))
            .for_each(|(curve, fb)| {
                let mode = if is_open { Mode::Open } else { Mode::Close };
                let curve = mode.regularize(curve);
                let efd = Efd2::from_curve_harmonic(curve, harmonic).unwrap();
                efd_stack.lock().unwrap().push(efd.coeffs().to_owned());
                let trans = arr1(&[efd.rot, efd.scale, efd.center[0], efd.center[1]]);
                trans_stack.lock().unwrap().push(trans);
                let mut stack = fb_stack.lock().unwrap();
                stack.push(arr1(&fb.vec()));
                callback(stack.len());
                inv_stack.lock().unwrap().push(arr0(fb.inv()));
            });
            if efd_stack.lock().unwrap().len() >= n {
                break;
            }
        }
        let fb = stack(fb_stack, n);
        let inv = stack(inv_stack, n);
        let efd = stack(efd_stack, n);
        let trans = stack(trans_stack, n);
        Self { fb, inv, efd, trans }
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
        impl_read!(r, fb, inv, efd, trans)
    }

    /// Write codebook to NPZ file.
    pub fn write(&self, w: impl Write + Seek) -> Result<(), ndarray_npy::WriteNpzError> {
        let mut w = ndarray_npy::NpzWriter::new_compressed(w);
        macro_rules! impl_write {
            ($w:ident, $($field:ident),+) => {
                let Self { $($field),+ } = self;
                $($w.add_array(stringify!($field), $field)?;)+
            };
        }
        impl_write!(w, fb, inv, efd, trans);
        w.finish().map(|_| ())
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
    pub fn fetch(&self, target: &[[f64; 2]], n: usize) -> Vec<(f64, FourBar)> {
        self.fetch_inner(target, n, true)
    }

    /// Get the nearest four-bar linkage from a target curve.
    pub fn fetch_1st(&self, target: &[[f64; 2]]) -> Option<(f64, FourBar)> {
        self.fetch_1st_inner(target, true)
    }

    /// Fetch without applying transformation.
    pub fn fetch_raw(&self, target: &[[f64; 2]], n: usize) -> Vec<(f64, FourBar)> {
        self.fetch_inner(target, n, false)
    }

    /// Fetch nearest without applying transformation.
    pub fn fetch_1st_raw(&self, target: &[[f64; 2]]) -> Option<(f64, FourBar)> {
        self.fetch_1st_inner(target, false)
    }

    fn fetch_inner(&self, target: &[[f64; 2]], n: usize, trans: bool) -> Vec<(f64, FourBar)> {
        if n == 1 {
            return self.fetch_1st_inner(target, trans).into_iter().collect();
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
            .take(n)
            .map(|i| (dis[i], self.pick(i, &target, trans)))
            .collect()
    }

    fn fetch_1st_inner(&self, target: &[[f64; 2]], trans: bool) -> Option<(f64, FourBar)> {
        let target = Efd2::from_curve_harmonic(target, self.harmonic()).unwrap();
        #[cfg(feature = "rayon")]
        let iter = self.efd.axis_iter(Axis(0)).into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = self.efd.axis_iter(Axis(0));
        iter.map(|efd| target.l1_norm(&Efd2::try_from_coeffs(efd.to_owned()).unwrap()))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, err)| (err, self.pick(i, &target, trans)))
    }

    fn pick(&self, i: usize, target: &Transform2, trans: bool) -> FourBar {
        let view = self.fb.slice(s![i, ..]);
        let inv = self.inv[i];
        let fb = NormFourBar::try_from_slice(view.as_slice().unwrap(), inv).unwrap();
        if trans {
            let trans = {
                let t = self.trans.slice(s![i, ..]);
                Transform2 { rot: t[0], scale: t[1], center: [t[2], t[3]] }
            };
            FourBar::from_trans(fb, &trans.to(target))
        } else {
            fb.into()
        }
    }

    /// Merge two data to one codebook.
    pub fn merge(&self, rhs: &Self) -> Self {
        let mut cb = self.clone();
        cb.merge_inplace(rhs);
        cb
    }

    /// Merge two data to one codebook inplace.
    pub fn merge_inplace(&mut self, rhs: &Self) {
        if self.is_empty() {
            self.clone_from(rhs);
        } else {
            macro_rules! merge {
                ($($field:ident),+) => {$(
                    self.$field = ndarray::concatenate![Axis(0), self.$field, rhs.$field];
                )+};
            }
            merge!(fb, inv, efd, trans);
        }
    }
}

impl FromIterator<Codebook> for Codebook {
    fn from_iter<T: IntoIterator<Item = Codebook>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|a, b| a.merge(&b))
            .unwrap_or_default()
    }
}
