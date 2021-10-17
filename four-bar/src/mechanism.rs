use crate::{FourBar, Point};
use rayon::prelude::*;
use std::{f64::consts::TAU, sync::Arc};

/// Modify the angle of four bar linkage.
fn four_bar_angle(angle: f64, formulas: &mut [Formula]) {
    if let Formula::Pla(_, _, ref mut a, _) = formulas[0] {
        *a = angle;
    } else {
        panic!("invalid four bar")
    }
}

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
    formulas: Vec<Formula>,
}

impl Mechanism {
    /// Create four bar linkages.
    pub fn four_bar(m: FourBar) -> Self {
        let joints = vec![
            [m.p0.x(), m.p0.y()],
            [m.p0.x() + m.l0 * m.a.cos(), m.p0.y() + m.l0 * m.a.sin()],
            [0., 0.],
            [0., 0.],
            [0., 0.],
        ];
        let mut formulas = Vec::with_capacity(3);
        formulas.push(Formula::Pla(0, m.l1, 0., 2));
        if (m.l0 - m.l2).abs() < 1e-20 && (m.l1 - m.l3).abs() < 1e-20 {
            // Special case
            formulas.push(Formula::Ppp(0, 2, 1, 3));
        } else {
            formulas.push(Formula::Pllp(2, m.l2, m.l3, 1, m.inv, 3));
        }
        formulas.push(Formula::Plap(2, m.l4, m.g, 3, 4));
        Self { joints, formulas }
    }

    /// A loop trajectory for only coupler point.
    pub fn four_bar_loop(&self, start: f64, n: usize) -> Vec<[f64; 2]> {
        let interval = TAU / n as f64;
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
    pub fn par_four_bar_loop(self: Arc<Self>, start: f64, n: usize) -> Vec<[f64; 2]> {
        let interval = TAU / n as f64;
        (0..n)
            .into_par_iter()
            .map(|i| {
                let four_bar = self.clone();
                let a = start + i as f64 * interval;
                let mut ans = [[0., 0.]];
                four_bar.apply(a, [4], &mut ans);
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
        let mut formulas = self.formulas.clone();
        four_bar_angle(angle, &mut formulas);
        for f in formulas.iter() {
            f.apply(&mut joints);
        }
        for (ans, joint) in ans.iter_mut().zip(joint) {
            *ans = joints[joint];
        }
    }

    /// Get the length of the joints.
    pub fn joint_len(&self) -> usize {
        self.joints.len()
    }
}
