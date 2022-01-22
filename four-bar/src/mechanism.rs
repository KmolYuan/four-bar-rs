use crate::{FourBar, Point};
use rayon::prelude::*;
use std::f64::consts::TAU;

#[derive(Clone)]
enum Formula {
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

/// Geometry constraint solver of the linkage mechanisms.
pub struct Mechanism {
    /// The joint positions.
    pub joints: Vec<[f64; 2]>,
    fs: Vec<Formula>,
}

impl Mechanism {
    /// Create four bar linkages.
    pub fn four_bar(m: &FourBar) -> Self {
        let joints = vec![
            [m.p0.x(), m.p0.y()],
            [m.p0.x() + m.l0 * m.a.cos(), m.p0.y() + m.l0 * m.a.sin()],
            [0., 0.],
            [0., 0.],
            [0., 0.],
        ];
        let mut fs = Vec::with_capacity(3);
        fs.push(Formula::Pla(0, m.l1, 0., 2));
        if (m.l0 - m.l2).abs() < 1e-20 && (m.l1 - m.l3).abs() < 1e-20 {
            // Special case
            fs.push(Formula::Ppp(0, 2, 1, 3));
        } else {
            fs.push(Formula::Pllp(2, m.l2, m.l3, 1, m.inv, 3));
        }
        fs.push(Formula::Plap(2, m.l4, m.g, 3, 4));
        Self { joints, fs }
    }

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
        let mut joints = self.joints.clone();
        let mut formulas = self.fs.clone();
        match formulas[0] {
            Formula::Pla(_, _, ref mut a, _) => *a = angle,
            _ => panic!("invalid four bar"),
        }
        for f in formulas {
            f.apply(&mut joints);
        }
        if joints.iter().any(|[x, y]| x.is_nan() || y.is_nan()) {
            ans.clone_from(&[[f64::NAN; 2]; N]);
        } else {
            for (ans, joint) in ans.iter_mut().zip(joint) {
                ans.clone_from(&joints[joint]);
            }
        }
    }

    /// Get the length of the joints.
    pub fn joint_len(&self) -> usize {
        self.joints.len()
    }
}
