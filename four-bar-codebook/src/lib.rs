#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use four_bar::{efd::Efd2, mh::utility::prelude::*, Mechanism, NormFourBar};
use ndarray_npy::{WriteNpyError, WriteNpyExt as _};
use std::{f64::consts::TAU, sync::Mutex};

/// Generate a codebook for close curve mechanism.
pub fn gen_codebook(
    fb_w: impl std::io::Write,
    efd_w: impl std::io::Write,
    n: usize,
    res: usize,
    harmonic: usize,
    open: bool,
) -> Result<(), WriteNpyError> {
    let rng = Rng::new(None);
    let fb_stack = Mutex::new(Vec::with_capacity(n));
    let stack = Mutex::new(Vec::with_capacity(n));
    loop {
        let len = stack.lock().unwrap().len();
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
                    stack.lock().unwrap().push(efd.unwrap());
                }
            })
        });
        if stack.lock().unwrap().len() >= n {
            break;
        }
    }
    let fb_stack = fb_stack.into_inner().unwrap();
    let stack = stack.into_inner().unwrap();
    let arrays = fb_stack.iter().take(n).map(Array::view).collect::<Vec<_>>();
    ndarray::stack(Axis(0), &arrays).unwrap().write_npy(fb_w)?;
    let arrays = stack.iter().take(n).map(Array::view).collect::<Vec<_>>();
    ndarray::stack(Axis(0), &arrays).unwrap().write_npy(efd_w)
}
