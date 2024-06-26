//! Create a atlas database for four-bar linkages.
pub use self::distr::{Code, Distr};
use super::{NormFourBar, SNormFourBar};
use mh::random::{Rng, SeedOpt};
#[cfg(feature = "rayon")]
use mh::rayon::prelude::*;
use ndarray::*;
pub use ndarray_npy::{ReadNpzError, WriteNpzError};
use std::{marker::PhantomData, sync::Mutex};

mod distr;

/// Planar four-bar atlas type.
pub type FbAtlas = Atlas<NormFourBar, 5, 2>;
/// Spherical four-bar atlas type.
pub type SFbAtlas = Atlas<SNormFourBar, 6, 3>;

fn to_arr<A, S, D>(stack: Mutex<Vec<ArrayBase<S, D>>>, n: usize) -> Array<A, D::Larger>
where
    A: Clone,
    S: ndarray::Data<Elem = A>,
    D: Dimension,
{
    let stack = stack.into_inner().unwrap();
    let arrays = stack.iter().take(n).map(|a| a.view()).collect::<Vec<_>>();
    ndarray::stack(Axis(0), &arrays).unwrap()
}

fn efd_to_arr<const D: usize>(efd: efd::Efd<D>) -> Array2<f64>
where
    efd::U<D>: efd::EfdDim<D>,
{
    let harmonic = efd.harmonic();
    let (m, _) = efd.into_inner();
    let v = m.into_iter().flat_map(|m| m.data.0).flatten().collect();
    // SAFETY: The shape is correct.
    unsafe { Array2::from_shape_vec_unchecked([harmonic, D * 2], v) }
}

fn arr_to_efd<const D: usize>(arr: ArrayView2<f64>) -> efd::Efd<D>
where
    efd::U<D>: efd::EfdDim<D>,
{
    let mut coeffs = Vec::with_capacity(arr.nrows());
    for m in arr.rows() {
        coeffs.push(efd::Kernel::from_iterator(m.iter().copied()));
    }
    efd::Efd::from_parts_unchecked(coeffs, efd::GeoVar::identity())
}

/// Atlas generation config.
#[derive(Clone)]
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
    pub seed: SeedOpt,
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
            seed: SeedOpt::Entropy,
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
        fn seed(SeedOpt)
    }
}

/// Atlas type.
pub struct Atlas<M, const N: usize, const D: usize> {
    fb: Array2<f64>,
    stat: Array1<u8>,
    efd: Array3<f64>,
    _marker: PhantomData<M>,
}

impl<M, const N: usize, const D: usize> Clone for Atlas<M, N, D> {
    fn clone(&self) -> Self {
        Self {
            fb: self.fb.clone(),
            stat: self.stat.clone(),
            efd: self.efd.clone(),
            _marker: PhantomData,
        }
    }
}

impl<M, const N: usize, const D: usize> Default for Atlas<M, N, D>
where
    M: Code<N, D>,
    efd::U<D>: efd::EfdDim<D>,
{
    fn default() -> Self {
        Self {
            fb: Array2::default([0, N]),
            stat: Array1::default(0),
            efd: Array3::default([0, 0, D * 2]),
            _marker: PhantomData,
        }
    }
}

impl<M, const N: usize, const D: usize> Atlas<M, N, D>
where
    M: Code<N, D>,
    efd::U<D>: efd::EfdDim<D>,
{
    /// Takes time to generate atlas data.
    pub fn make(cfg: Cfg) -> Self
    where
        M: Send,
        [f64; D]: Sync + Send,
    {
        Self::make_with(cfg, |_| ())
    }

    /// Takes time to generate atlas data with a callback function.
    pub fn make_with<CB>(cfg: Cfg, callback: CB) -> Self
    where
        M: Send,
        CB: Fn(usize) + Sync + Send,
        [f64; D]: Sync + Send,
    {
        let Cfg { is_open, size, res, harmonic, seed } = cfg;
        let mut rng = Rng::new(seed);
        let fb_stack = Mutex::new(Vec::with_capacity(size));
        let stat_stack = Mutex::new(Vec::with_capacity(size));
        let efd_stack = Mutex::new(Vec::with_capacity(size));
        loop {
            let len = efd_stack.lock().unwrap().len();
            let n = (size - len) / 2;
            #[cfg(not(feature = "rayon"))]
            let iter = rng.stream(n).into_iter();
            #[cfg(feature = "rayon")]
            let iter = rng.stream(n).into_par_iter();
            iter.flat_map(|mut rng| rng.sample(Distr::<M, N>::new()))
                .filter_map(|fb| fb.get_curve(res, is_open).map(|c| (c, fb)))
                .filter(|(c, _)| c.len() > 1)
                .for_each(|(curve, fb)| {
                    let efd = efd::Efd::from_curve_harmonic(curve, is_open, harmonic);
                    efd_stack.lock().unwrap().push(efd_to_arr(efd));
                    let (code, stat) = fb.to_code();
                    let mut stack = fb_stack.lock().unwrap();
                    stack.push(arr1(&code));
                    callback(stack.len());
                    stat_stack.lock().unwrap().push(arr0(stat));
                });
            if efd_stack.lock().unwrap().len() >= size {
                break;
            }
        }
        let fb = to_arr(fb_stack, size);
        let stat = to_arr(stat_stack, size);
        let efd = to_arr(efd_stack, size);
        Self { fb, stat, efd, _marker: PhantomData }
    }

    /// Read atlas from NPZ file.
    pub fn read<R>(r: R) -> Result<Self, ReadNpzError>
    where
        R: std::io::Read + std::io::Seek,
    {
        let mut r = ndarray_npy::NpzReader::new(r)?;
        macro_rules! impl_read {
            ($r:ident, $($field:ident),+) => {{
                $(let $field = $r.by_name(stringify!($field))?;)+
                Self { $($field),+, _marker: PhantomData }
            }};
        }
        macro_rules! impl_check {
            ($actual:expr, $expect:expr) => {
                let actual = $actual;
                let expect = $expect;
                if actual != expect {
                    return Err(ReadNpzError::Npy(ndarray_npy::ReadNpyError::WrongNdim(
                        Some(expect),
                        actual,
                    )));
                }
            };
        }
        let atlas = impl_read!(r, fb, stat, efd);
        impl_check!(atlas.fb.len_of(Axis(1)), N);
        impl_check!(atlas.efd.len_of(Axis(2)), D * 2);
        Ok(atlas)
    }

    /// Iterate over the normalized linkages.
    pub fn fb_norm_iter(&self) -> impl Iterator<Item = M> + '_ {
        std::iter::zip(&self.stat, self.fb.rows())
            .map(|(stat, arr)| M::from_code(arr.as_slice().unwrap(), *stat))
    }

    /// Iterate over the EFD coefficients.
    pub fn efd_iter(&self) -> impl Iterator<Item = efd::Efd<D>> + '_ {
        self.efd.axis_iter(Axis(0)).map(arr_to_efd)
    }

    /// Get the n-nearest four-bar linkages from a target curve.
    ///
    /// This method will keep the dimensional variables without transform.
    #[allow(clippy::type_complexity)]
    pub fn fetch_raw(
        &self,
        target: &[[f64; D]],
        is_open: bool,
        size: usize,
    ) -> Option<((f64, M::De), Vec<(f64, M)>)>
    where
        efd::Efd<D>: Sync,
    {
        (!self.is_empty()).then_some(())?;
        let res = target.len();
        let target = efd::Efd::from_curve_harmonic(target, is_open, self.harmonic());
        let geo = target.as_geo();
        #[cfg(not(feature = "rayon"))]
        let iter = self.efd.axis_iter(Axis(0));
        #[cfg(feature = "rayon")]
        let iter = self.efd.axis_iter(Axis(0)).into_par_iter();
        let dis = iter
            .map(|arr| target.err(&arr_to_efd(arr)))
            .collect::<Vec<_>>();
        if size == 1 {
            let (first_i, err) = dis
                .into_iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())?;
            let first = (err, self.pick(first_i, geo, is_open, res));
            let pool = vec![(err, self.pick_norm(first_i))];
            return Some((first, pool));
        }
        let mut ind = (0..dis.len()).collect::<Vec<_>>();
        ind.sort_by(|&a, &b| dis[a].partial_cmp(&dis[b]).unwrap());
        let first_i = *ind.first()?;
        let first = (dis[first_i], self.pick(first_i, geo, is_open, res));
        let pool = ind
            .into_iter()
            .take(size)
            .map(|i| (dis[i], self.pick_norm(i)))
            .collect();
        Some((first, pool))
    }

    /// Get the nearest four-bar linkage from a target curve.
    pub fn fetch_1st(&self, target: &[[f64; D]], is_open: bool) -> Option<(f64, M::De)>
    where
        efd::Efd<D>: Sync,
    {
        self.fetch_raw(target, is_open, 1).map(|(first, _)| first)
    }

    /// Get the n-nearest four-bar linkages from a target curve.
    ///
    /// Slower than [`Self::fetch_1st()`].
    pub fn fetch(&self, target: &[[f64; D]], is_open: bool, size: usize) -> Vec<(f64, M::De)>
    where
        efd::Efd<D>: Sync,
    {
        if self.is_empty() {
            return Vec::new();
        } else if size == 1 {
            return Vec::from_iter(self.fetch_1st(target, is_open));
        }
        let res = target.len();
        let target = efd::Efd::from_curve_harmonic(target, is_open, self.harmonic());
        #[cfg(not(feature = "rayon"))]
        let iter = self.efd.axis_iter(Axis(0));
        #[cfg(feature = "rayon")]
        let iter = self.efd.axis_iter(Axis(0)).into_par_iter();
        let dis = iter
            .map(|arr| target.err(&arr_to_efd(arr)))
            .collect::<Vec<_>>();
        let mut ind = (0..self.len()).collect::<Vec<_>>();
        ind.sort_by(|&a, &b| dis[a].partial_cmp(&dis[b]).unwrap());
        ind.into_iter()
            .take(size)
            .map(|i| (dis[i], self.pick(i, target.as_geo(), is_open, res)))
            .collect()
    }

    fn pick_norm(&self, i: usize) -> M {
        M::from_code(self.fb.row(i).as_slice().unwrap(), self.stat[i])
    }

    fn pick(
        &self,
        i: usize,
        geo: &efd::GeoVar<efd::Rot<D>, D>,
        is_open: bool,
        res: usize,
    ) -> M::De {
        let fb = self.pick_norm(i);
        let curve = fb.get_curve(res, is_open).unwrap();
        let efd = efd::Efd::from_curve(curve, is_open);
        fb.trans_denorm(&efd.as_geo().to(geo))
    }
}

impl<M, const N: usize, const D: usize> Atlas<M, N, D> {
    /// Write atlas to NPZ file.
    pub fn write<W>(&self, w: W) -> Result<(), WriteNpzError>
    where
        W: std::io::Write + std::io::Seek,
    {
        let mut w = ndarray_npy::NpzWriter::new_compressed(w);
        macro_rules! impl_write {
            ($w:ident, $($field:ident),+) => {
                let Self { $($field),+, _marker: _ } = self;
                $($w.add_array(stringify!($field), $field)?;)+
            };
        }
        impl_write!(w, fb, stat, efd);
        w.finish()?;
        Ok(())
    }

    /// Clear the atlas.
    pub fn clear(&mut self)
    where
        Self: Default,
    {
        *self = Self::default();
    }

    /// Length, total size.
    #[inline]
    pub fn len(&self) -> usize {
        self.fb.nrows()
    }

    /// Return whether the atlas has any data.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Number of the harmonics.
    #[inline]
    pub fn harmonic(&self) -> usize {
        self.efd.len_of(Axis(1))
    }

    /// Get a reference to the data.
    ///
    /// Data is stored in a 2D array, each row is a linkage code.
    pub fn fb_data(&self) -> &Array2<f64> {
        &self.fb
    }

    /// Iterate over with an "open state" of the linkages.
    pub fn is_open_iter(&self) -> impl Iterator<Item = bool> + '_ {
        (self.efd.axis_iter(Axis(0))).map(|efd| efd.slice(s![.., D..]).sum() == 0.)
    }

    /// Merge two data to one atlas.
    pub fn merge(mut self, rhs: Self) -> Result<Self, ndarray::ShapeError> {
        self.merge_inplace(rhs)?;
        Ok(self)
    }

    /// Merge two data to one atlas inplace.
    pub fn merge_inplace(&mut self, mut rhs: Self) -> Result<(), ndarray::ShapeError> {
        self.fb.append(Axis(0), rhs.fb.view())?;
        self.stat.append(Axis(0), rhs.stat.view())?;
        // Extend the harmonic number (zero padding) if needed
        macro_rules! padding {
            ($lhs:ident, $rhs:ident) => {{
                let mut shape = $lhs.efd.raw_dim();
                shape[1] = $rhs.harmonic();
                ($lhs.efd.append(Axis(1), Array::zeros(shape).view()))
                    .unwrap_or_else(|_| unreachable!());
            }};
        }
        match self.harmonic().cmp(&rhs.harmonic()) {
            std::cmp::Ordering::Less => padding!(self, rhs),
            std::cmp::Ordering::Greater => padding!(rhs, self),
            std::cmp::Ordering::Equal => (),
        }
        self.efd.append(Axis(0), rhs.efd.view())
    }
}

impl<M, const N: usize, const D: usize> FromIterator<Self> for Atlas<M, N, D>
where
    M: Code<N, D> + Send,
    efd::U<D>: efd::EfdDim<D>,
{
    /// Merge multiple data to one atlas.
    ///
    /// # Panics
    /// Panics if the data shape is not correct.
    fn from_iter<T: IntoIterator<Item = Self>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|a, b| a.merge(b).expect("Shape mismatched"))
            .unwrap_or_default()
    }
}
