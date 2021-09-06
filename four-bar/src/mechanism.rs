use crate::Point;
use std::{
    f64::consts::TAU,
    io::{Error, ErrorKind, Result},
};

enum Formula {
    Pla(usize, f64, f64, usize),
    Plap(usize, f64, f64, usize, usize),
    Pllp(usize, f64, f64, usize, bool, usize),
    Ppp(usize, usize, usize, usize),
}

impl Formula {
    fn apply(&self, joints: &mut Vec<impl Point>) {
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

pub struct Mechanism {
    /// The joint positions.
    pub joints: Vec<[f64; 2]>,
    formulas: Vec<Formula>,
}

impl Mechanism {
    /// Create four bar linkages.
    pub fn four_bar(
        (p0x, p0y, a): (f64, f64, f64),
        l0: f64,
        l1: f64,
        l2: f64,
        l3: f64,
        l4: f64,
        g: f64,
    ) -> Self {
        let mut joints = Vec::with_capacity(5);
        joints.push([p0x, p0y]);
        joints.push([p0x + l0 * a.cos(), p0y + l0 * a.sin()]);
        for _ in 2..5 {
            joints.push([0., 0.]);
        }
        let mut formulas = Vec::with_capacity(3);
        formulas.push(Formula::Pla(0, l1, 0., 2));
        if (l0 - l2).abs() < 1e-20 && (l1 - l3).abs() < 1e-20 {
            // Special case
            formulas.push(Formula::Ppp(0, 2, 1, 3));
        } else {
            formulas.push(Formula::Pllp(2, l2, l3, 1, false, 3));
        }
        formulas.push(Formula::Plap(2, l4, g, 3, 4));
        Self { joints, formulas }
    }

    /// Modify the angle of four bar linkage.
    pub fn four_bar_angle(&mut self, angle: f64) -> Result<()> {
        if let Formula::Pla(_, _, ref mut a, _) = self.formulas[0] {
            *a = angle;
        } else {
            return Err(Error::new(ErrorKind::AddrNotAvailable, "invalid four bar"));
        }
        self.apply();
        Ok(())
    }

    /// A loop trajectory for only coupler point.
    pub fn four_bar_loop(&mut self, start: f64, n: usize) -> Vec<[f64; 2]> {
        let interval = TAU / n as f64;
        let mut path = vec![[0.; 2]; n];
        for (i, c) in path.iter_mut().enumerate() {
            let a = start + i as f64 * interval;
            self.four_bar_angle(a).unwrap();
            *c = self.joints[4];
        }
        path
    }

    /// A loop trajectory for all moving pivot.
    pub fn four_bar_loop_all(&mut self, start: f64, n: usize) -> [Vec<[f64; 2]>; 3] {
        let interval = TAU / n as f64;
        let mut path = [vec![[0.; 2]; n], vec![[0.; 2]; n], vec![[0.; 2]; n]];
        for i in 0..n {
            let a = start + i as f64 * interval;
            self.four_bar_angle(a).unwrap();
            let mut failed = false;
            for j in (0..3).rev() {
                if self.joints[j + 2][0].is_nan() {
                    failed = true;
                }
                if failed {
                    path[j][i] = [f64::NAN, f64::NAN];
                } else {
                    path[j][i] = self.joints[j + 2];
                }
            }
        }
        path
    }

    fn apply(&mut self) {
        for f in self.formulas.iter() {
            f.apply(&mut self.joints);
        }
    }

    /// Get the length of the joints.
    pub fn joint_len(&self) -> usize {
        self.joints.len()
    }
}
