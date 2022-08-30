#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use four_bar::{efd::Efd2, mh::utility::prelude::*, FourBar, Mechanism, NormFourBar};
use std::{
    io::{Read, Seek, Write},
    sync::Mutex,
};

/// Codebook type.
pub struct CodeBook {
    open: Array0<bool>,
    fb: Array2<f64>,
    efd: Array3<f64>,
}

impl CodeBook {
    /// Takes time to generate codebook data.
    pub fn make(open: bool, n: usize, res: usize, harmonic: usize) -> Self {
        let rng = Rng::new(None);
        let fb_stack = Mutex::new(Vec::with_capacity(n));
        let efd_stack = Mutex::new(Vec::with_capacity(n));
        loop {
            let len = efd_stack.lock().unwrap().len();
            (0..(n - len) / 2).into_par_iter().for_each(|_| {
                let v = [
                    rng.float(1e-4..10.),
                    rng.float(1e-4..10.),
                    rng.float(1e-4..10.),
                    rng.float(1e-4..10.),
                    rng.float(0.0..std::f64::consts::TAU),
                ];
                [false, true].into_par_iter().for_each(|inv| {
                    let fb = NormFourBar::from_vec(v, inv).to_close_curve();
                    let fb = match open {
                        false => fb.to_close_curve(),
                        true => fb.to_open_curve(),
                    };
                    if open != fb.ty().is_open_curve() {
                        return;
                    }
                    if let Some([start, end]) = fb.angle_bound() {
                        let curve = Mechanism::new(&fb).curve(start, end, res);
                        let efd = Efd2::from_curve(&curve, harmonic);
                        fb_stack.lock().unwrap().push(arr1(&fb.v));
                        efd_stack.lock().unwrap().push(efd.unwrap());
                    }
                })
            });
            if efd_stack.lock().unwrap().len() >= n {
                break;
            }
        }
        let fb = fb_stack.into_inner().unwrap();
        let efd = efd_stack.into_inner().unwrap();
        let arrays = fb.iter().take(n).map(Array::view).collect::<Vec<_>>();
        let fb = ndarray::stack(Axis(0), &arrays).unwrap();
        let arrays = efd.iter().take(n).map(Array::view).collect::<Vec<_>>();
        let efd = ndarray::stack(Axis(0), &arrays).unwrap();
        let open = arr0(open);
        Self { open, fb, efd }
    }

    /// Read codebook from NPZ file.
    pub fn read(r: impl Read + Seek) -> Result<Self, ndarray_npy::ReadNpzError> {
        let mut r = ndarray_npy::NpzReader::new(r)?;
        let open = r.by_name("open")?;
        let fb = r.by_name("fb")?;
        let efd = r.by_name("efd")?;
        Ok(Self { open, fb, efd })
    }

    /// Write codebook to NPZ file.
    pub fn write(&self, w: impl Write + Seek) -> Result<(), ndarray_npy::WriteNpzError> {
        let mut w = ndarray_npy::NpzWriter::new(w);
        w.add_array("open", &self.open)?;
        w.add_array("fb", &self.fb)?;
        w.add_array("efd", &self.efd)?;
        w.finish().map(|_| ())
    }

    /// Return true if the codebook saves open curves.
    pub fn is_open(&self) -> bool {
        *self.open.first().unwrap()
    }

    /// Total size.
    pub fn size(&self) -> usize {
        self.fb.nrows()
    }

    /// Number of the harmonics.
    pub fn harmonic(&self) -> usize {
        self.efd.len_of(Axis(1))
    }

    /// Get the nearest four-bar linkage from a target curve.
    pub fn fetch(&self, target: &[[f64; 2]]) -> Vec<FourBar> {
        let target = Efd2::from_curve(target, self.harmonic());
        let _dis = self
            .efd
            .axis_iter(Axis(0))
            .into_par_iter()
            .map(|efd| target.manhattan(&Efd2::try_from_coeffs(efd.to_owned()).unwrap()))
            .collect::<Vec<_>>();
        todo!()
    }
}
