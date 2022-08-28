#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use four_bar::{efd::Efd2, mh::utility::prelude::*, Mechanism, NormFourBar};
use ndarray_npy::{WriteNpyError, WriteNpyExt as _};
use std::{f64::consts::TAU, sync::Mutex};

/// Option type.
pub struct Opt {
    /// Number of the dataset
    pub n: usize,
    /// Curve resolution
    pub res: usize,
    /// Number of EFD harmonics
    pub harmonic: usize,
    /// Is open curve?
    pub open: bool,
}

/// Codebook type.
pub struct CodeBook {
    fb: Array2<f64>,
    efd: Array3<f64>,
}

impl CodeBook {
    /// Generate codebook data.
    pub fn generate(opt: Opt) -> Self {
        let Opt { n, res, harmonic, open } = opt;
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
                    rng.float(0.0..TAU),
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
        Self { fb, efd }
    }

    /// Write codebook to NPY files.
    pub fn write<W1, W2>(&self, fb_w: W1, efd_w: W2) -> Result<(), WriteNpyError>
    where
        W1: std::io::Write,
        W2: std::io::Write,
    {
        self.fb.write_npy(fb_w)?;
        self.efd.write_npy(efd_w)
    }
}
