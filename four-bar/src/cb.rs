//! Create a codebook database for four-bar linkages.
pub use self::distr::Code;
use super::{NormFourBar, SNormFourBar};
use mh::{
    random::{Rng, SeedOption},
    utility::prelude::*,
};
pub use ndarray_npy::{ReadNpzError, WriteNpzError};
use std::{marker::PhantomData, sync::Mutex};

mod distr;

/// Planar four-bar codebook type.
pub type FbCodebook = Codebook<NormFourBar, efd::D2, 5>;
/// Spherical four-bar codebook type.
pub type SFbCodebook = Codebook<SNormFourBar, efd::D3, 6>;

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

fn efd_to_arr<D: efd::EfdDim>(efd: efd::Efd<D>) -> Array2<f64> {
    let mat = efd.into_inner();
    unsafe { Array2::from_shape_vec_unchecked([mat.ncols(), mat.nrows()], mat.data.into()) }
}

fn arr_to_efd<D: efd::EfdDim>(arr: ArrayView2<f64>) -> efd::Efd<D> {
    use efd::na::{self, DimName as _};
    let vec = arr.to_owned().into_raw_vec();
    let data = na::VecStorage::new(efd::CDim::<D>::name(), na::Dyn(arr.nrows()), vec);
    unsafe { efd::Efd::from_coeffs_unchecked(efd::Coeff::<D>::from_data(data)) }
}

/// Codebook generation config.
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
pub struct Codebook<C, D, const N: usize>
where
    C: Code<D, N>,
    D: efd::EfdDim,
{
    fb: Array2<f64>,
    inv: Array1<bool>,
    efd: Array3<f64>,
    _marker: PhantomData<(C, D)>,
}

impl<C, D, const N: usize> Clone for Codebook<C, D, N>
where
    C: Code<D, N>,
    D: efd::EfdDim,
{
    fn clone(&self) -> Self {
        Self {
            fb: self.fb.clone(),
            inv: self.inv.clone(),
            efd: self.efd.clone(),
            _marker: PhantomData,
        }
    }
}

impl<C, D, const N: usize> Default for Codebook<C, D, N>
where
    C: Code<D, N>,
    D: efd::EfdDim,
{
    fn default() -> Self {
        Self {
            fb: Array2::default([0, N]),
            inv: Array1::default(0),
            efd: Array3::default([0, 0, <D::Trans as efd::Trans>::dim() * 2]),
            _marker: PhantomData,
        }
    }
}

impl<C, D, const N: usize> Codebook<C, D, N>
where
    C: Code<D, N> + Send,
    D: efd::EfdDim,
{
    /// Takes time to generate codebook data.
    pub fn make(cfg: Cfg) -> Self
    where
        efd::Coord<D>: Sync + Send,
    {
        Self::make_with(cfg, |_| ())
    }

    /// Takes time to generate codebook data with a callback function.
    pub fn make_with<CB>(cfg: Cfg, callback: CB) -> Self
    where
        CB: Fn(usize) + Sync + Send,
        efd::Coord<D>: Sync + Send,
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
            iter.flat_map(|rng| rng.sample(C::distr()))
                .filter_map(|fb| fb.get_curve(res, is_open).map(|c| (c, fb)))
                .filter(|(c, _)| c.len() > 1)
                .for_each(|(curve, fb)| {
                    let efd = efd::Efd::<D>::from_curve_harmonic(curve, is_open, harmonic);
                    efd_stack.lock().unwrap().push(efd_to_arr(efd));
                    let (code, inv) = fb.to_code();
                    let mut stack = fb_stack.lock().unwrap();
                    stack.push(arr1(&code));
                    callback(stack.len());
                    inv_stack.lock().unwrap().push(arr0(inv));
                });
            if efd_stack.lock().unwrap().len() >= size {
                break;
            }
        }
        let fb = to_arr(fb_stack, size);
        let inv = to_arr(inv_stack, size);
        let efd = to_arr(efd_stack, size);
        Self { fb, inv, efd, _marker: PhantomData }
    }

    /// Read codebook from NPZ file.
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
        let cb = impl_read!(r, fb, inv, efd);
        impl_check!(cb.fb.len_of(Axis(1)), N);
        impl_check!(cb.efd.len_of(Axis(2)), <D::Trans as efd::Trans>::dim() * 2);
        Ok(cb)
    }

    /// Write codebook to NPZ file.
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
        impl_write!(w, fb, inv, efd);
        w.finish()?;
        Ok(())
    }

    /// Length, total size.
    pub fn len(&self) -> usize {
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
    pub fn fetch_raw(&self, target: &[efd::Coord<D>], is_open: bool, size: usize) -> Vec<(f64, C)>
    where
        efd::Efd<D>: Sync,
    {
        if self.is_empty() {
            return Vec::new();
        }
        let target = efd::Efd::<D>::from_curve_harmonic(target, is_open, self.harmonic());
        #[cfg(feature = "rayon")]
        let iter = self.efd.axis_iter(Axis(0)).into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = self.efd.axis_iter(Axis(0));
        let dis = iter
            .map(|arr| target.l2_norm(&arr_to_efd(arr)))
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
    pub fn fetch_1st(
        &self,
        target: &[efd::Coord<D>],
        is_open: bool,
        res: usize,
    ) -> Option<(f64, C::De)>
    where
        efd::Efd<D>: Sync,
    {
        if self.is_empty() {
            return None;
        }
        let target = efd::Efd::<D>::from_curve_harmonic(target, is_open, self.harmonic());
        #[cfg(feature = "rayon")]
        let iter = self.efd.axis_iter(Axis(0)).into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = self.efd.axis_iter(Axis(0));
        iter.map(|arr| target.l2_norm(&arr_to_efd(arr)))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, err)| (err, self.pick(i, target.as_trans(), is_open, res)))
    }

    /// Get the n-nearest four-bar linkages from a target curve.
    ///
    /// Slower than [`Self::fetch_1st()`].
    pub fn fetch(
        &self,
        target: &[efd::Coord<D>],
        is_open: bool,
        size: usize,
        res: usize,
    ) -> Vec<(f64, C::De)>
    where
        efd::Efd<D>: Sync,
    {
        if self.is_empty() {
            return Vec::new();
        } else if size == 1 {
            return self.fetch_1st(target, is_open, res).into_iter().collect();
        }
        let target = efd::Efd::<D>::from_curve_harmonic(target, is_open, self.harmonic());
        #[cfg(feature = "rayon")]
        let iter = self.efd.axis_iter(Axis(0)).into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = self.efd.axis_iter(Axis(0));
        let dis = iter
            .map(|arr| target.l2_norm(&arr_to_efd(arr)))
            .collect::<Vec<_>>();
        let mut ind = (0..self.len()).collect::<Vec<_>>();
        ind.sort_by(|&a, &b| dis[a].partial_cmp(&dis[b]).unwrap());
        ind.into_iter()
            .take(size)
            .map(|i| (dis[i], self.pick(i, target.as_trans(), is_open, res)))
            .collect()
    }

    fn pick_norm(&self, i: usize) -> C {
        let code = self
            .fb
            .slice(s![i, ..])
            .as_slice()
            .unwrap()
            .try_into()
            .unwrap();
        C::from_code(code, self.inv[i])
    }

    fn pick(&self, i: usize, trans: &efd::Transform<D::Trans>, is_open: bool, res: usize) -> C::De {
        let fb = self.pick_norm(i);
        let curve = fb.get_curve(res, is_open).unwrap();
        let efd = efd::Efd::<D>::from_curve(curve, is_open);
        fb.trans_denorm(&efd.as_trans().to(trans))
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

impl<C, D, const N: usize> FromIterator<Self> for Codebook<C, D, N>
where
    C: Code<D, N> + Send,
    D: efd::EfdDim,
{
    fn from_iter<T: IntoIterator<Item = Self>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|a, b| a.merge(&b).unwrap_or(a))
            .unwrap_or_default()
    }
}
