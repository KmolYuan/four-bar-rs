use crate::{FourBar, Point};
use rayon::prelude::*;
use std::{f64::consts::TAU, marker::PhantomData};

/// Mechanism position formula.
#[allow(missing_docs)]
#[derive(Clone)]
pub enum Formula {
    Pla(usize, f64, f64, usize),
    Plap(usize, f64, f64, usize, usize),
    Pllp(usize, f64, f64, usize, bool, usize),
    Ppp(usize, usize, usize, usize),
}

impl Formula {
    fn apply(&self, joints: &mut [impl Point]) {
        match *self {
            Self::Pla(c1, d0, a0, t) => {
                joints[t] = joints[c1].pla(d0, a0);
            }
            Self::Plap(c1, d0, a0, c2, t) => {
                joints[t] = joints[c1].plap(d0, a0, &joints[c2]);
            }
            Self::Pllp(c1, d0, d1, c2, inv, t) => {
                joints[t] = joints[c1].pllp(d0, d1, &joints[c2], inv);
            }
            Self::Ppp(c1, c2, c3, t) => {
                joints[t] = joints[c1].ppp(&joints[c2], &joints[c3]);
            }
        }
    }
}

/// A linkage type.
pub trait Linkage {
    /// Memory layout of the joints
    type Joint;

    /// Allocate memory for [`Mechanism`] type.
    fn allocate(&self) -> (Self::Joint, Vec<Formula>);
}

/// Geometry constraint solver of the linkage mechanisms.
pub struct Mechanism<L: Linkage> {
    /// The joint positions.
    pub joints: L::Joint,
    fs: Vec<Formula>,
    _marker: PhantomData<L>,
}

impl<L: Linkage> Mechanism<L> {
    /// Create a mechanism for the linkage.
    pub fn new(m: &L) -> Self {
        let (joints, fs) = m.allocate();
        Self {
            joints,
            fs,
            _marker: PhantomData,
        }
    }
}

impl Mechanism<FourBar> {
    /// A loop trajectory for only coupler point.
    pub fn four_bar_loop(&self, start: f64, end: f64, n: usize) -> Vec<[f64; 2]> {
        let interval = (end - start) / n as f64;
        let mut path = vec![[0.; 2]; n];
        for (i, c) in path.iter_mut().enumerate() {
            let a = start + i as f64 * interval;
            let mut ans = [[0., 0.]];
            self.apply(a, [4], &mut ans);
            *c = ans[0];
        }
        path
    }

    /// Get the trajectory by parallel computing.
    pub fn par_four_bar_loop(&self, start: f64, end: f64, n: usize) -> Vec<[f64; 2]> {
        let interval = (end - start) / n as f64;
        (0..n)
            .into_par_iter()
            .map(|i| {
                let a = start + i as f64 * interval;
                let mut ans = [[0., 0.]];
                self.apply(a, [4], &mut ans);
                ans[0]
            })
            .collect()
    }

    /// A loop trajectory for all moving pivot.
    pub fn four_bar_loop_all(&self, start: f64, n: usize) -> [Vec<[f64; 2]>; 3] {
        let interval = TAU / n as f64;
        let mut path = [vec![[0.; 2]; n], vec![[0.; 2]; n], vec![[0.; 2]; n]];
        for i in 0..n {
            let a = start + i as f64 * interval;
            let mut ans = [[0., 0.]; 3];
            self.apply(a, [2, 3, 4], &mut ans);
            for (path, ans) in path.iter_mut().zip(ans) {
                path[i] = ans;
            }
        }
        path
    }

    /// Calculate the formula, and write the answer into provided array.
    pub fn apply<const N: usize>(&self, angle: f64, joint: [usize; N], ans: &mut [[f64; 2]; N]) {
        let mut joints = self.joints;
        let mut formulas = self.fs.clone();
        match formulas.first_mut() {
            Some(Formula::Pla(_, _, ref mut a, _)) => *a = angle,
            _ => panic!("invalid four bar"),
        }
        for f in formulas {
            f.apply(&mut joints);
        }
        if joints[4][0].is_nan() || joints[4][1].is_nan() {
            ans.clone_from(&[[f64::NAN; 2]; N]);
        } else {
            for (ans, joint) in ans.iter_mut().zip(joint) {
                ans.clone_from(&joints[joint]);
            }
        }
    }
}
