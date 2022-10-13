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
    pub fn make(open: bool, n: usize, res: usize, harmonic: usize) -> Self {
        Self::make_with(open, n, res, harmonic, |_| ())
    }

    /// Takes time to generate codebook data with a callback function.
    pub fn make_with<C>(open: bool, n: usize, res: usize, harmonic: usize, callback: C) -> Self
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
            .map(|fb| match open {
                false => fb.to_close_curve(),
                true => fb.to_open_curve(),
            })
            .filter(|fb| open == fb.ty().is_open_curve())
            .filter_map(|fb| fb.curve(res).map(|c| (c, fb)))
            .for_each(|(curve, fb)| {
                let mode = if open { Mode::Open } else { Mode::Close };
                let curve = mode.regularize(curve);
                let efd = Efd2::from_curve_harmonic(curve, harmonic).unwrap();
                efd_stack.lock().unwrap().push(efd.coeffs().to_owned());
                let trans = arr1(&[efd.rot, efd.scale, efd.center[0], efd.center[1]]);
                trans_stack.lock().unwrap().push(trans);
                let mut stack = fb_stack.lock().unwrap();
                stack.push(arr1(&fb.v));
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

    /// Number of the harmonics.
    pub fn harmonic(&self) -> usize {
        self.efd.len_of(Axis(1))
    }

    /// Get the n-nearest four-bar linkages from a target curve.
    pub fn fetch(&self, target: &[[f64; 2]], n: usize) -> Vec<(f64, FourBar)> {
        if n == 1 {
            return self.fetch_1st(target).into_iter().collect();
        }
        let target = Efd2::from_curve_harmonic(target, self.harmonic()).unwrap();
        let dis = self
            .efd
            .axis_iter(Axis(0))
            .into_par_iter()
            .map(|efd| target.l1_norm(&Efd2::try_from_coeffs(efd.to_owned()).unwrap()))
            .collect::<Vec<_>>();
        let mut ind = (0..self.size()).collect::<Vec<_>>();
        ind.sort_by(|&a, &b| dis[a].partial_cmp(&dis[b]).unwrap());
        ind.into_iter()
            .take(n)
            .map(|i| (dis[i], self.pick(i, &target)))
            .collect()
    }

    /// Get the nearest four-bar linkage from a target curve.
    pub fn fetch_1st(&self, target: &[[f64; 2]]) -> Option<(f64, FourBar)> {
        let target = Efd2::from_curve_harmonic(target, self.harmonic()).unwrap();
        self.efd
            .axis_iter(Axis(0))
            .into_par_iter()
            .map(|efd| target.l1_norm(&Efd2::try_from_coeffs(efd.to_owned()).unwrap()))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, err)| (err, self.pick(i, &target)))
    }

    fn pick(&self, i: usize, target: &Transform2) -> FourBar {
        let view = self.fb.slice(s![i, ..]);
        let inv = self.inv[i];
        let fb = NormFourBar::try_from_slice(view.as_slice().unwrap(), inv).unwrap();
        let trans = {
            let t = self.trans.slice(s![i, ..]);
            Transform2 { rot: t[0], scale: t[1], center: [t[2], t[3]] }
        };
        FourBar::from_trans(fb, &trans.to(target))
    }

    /// Merge two data to one codebook.
    pub fn merge(&self, rhs: &Self) -> Self {
        Self {
            fb: ndarray::concatenate![Axis(0), self.fb, rhs.fb],
            inv: ndarray::concatenate![Axis(0), self.inv, rhs.inv],
            efd: ndarray::concatenate![Axis(0), self.efd, rhs.efd],
            trans: ndarray::concatenate![Axis(0), self.trans, rhs.trans],
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