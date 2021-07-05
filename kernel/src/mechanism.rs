use std::{
    f64::consts::TAU,
    io::{Error, ErrorKind, Result},
};

enum Formula {
    PLA(usize, f64, f64, usize),
    PLAP(usize, f64, f64, usize, usize),
    PLLP(usize, f64, f64, usize, bool, usize),
}

impl Formula {
    fn apply(&mut self, joints: &mut Vec<impl Point>) {
        match self {
            Self::PLA(c1, d0, a0, t) => {
                joints[*t] = joints[*c1].pla(*d0, *a0);
            }
            Self::PLAP(c1, d0, a0, c2, t) => {
                joints[*t] = joints[*c1].plap(*d0, *a0, &joints[*c2]);
            }
            Self::PLLP(c1, d0, d1, c2, inv, t) => {
                joints[*t] = joints[*c1].pllp(*d0, *d1, &joints[*c2], *inv);
            }
        }
    }
}

pub struct Mechanism {
    pub joints: Vec<[f64; 2]>,
    formulas: Vec<Formula>,
}

impl Mechanism {
    /// Create four bar linkages.
    pub fn four_bar(
        p0: (f64, f64),
        a: f64,
        l0: f64,
        l1: f64,
        l2: f64,
        l3: f64,
        l4: f64,
        g: f64,
    ) -> Self {
        let mut s = Self {
            joints: Vec::with_capacity(5),
            formulas: vec![],
        };
        s.joints.push([p0.0, p0.1]);
        s.joints.push([p0.0 + l0 * a.cos(), p0.1 + l0 * a.sin()]);
        for _ in 2..5 {
            s.joints.push([0., 0.]);
        }
        s.formulas.push(Formula::PLA(0, l1, 0., 2));
        s.formulas.push(Formula::PLLP(2, l2, l3, 1, false, 3));
        s.formulas.push(Formula::PLAP(2, l4, g, 3, 4));
        s
    }

    /// Modify the angle of four bar linkage.
    pub fn four_bar_angle(&mut self, angle: f64) -> Result<()> {
        if let Formula::PLA(_, _, ref mut a, _) = self.formulas[0] {
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
        for i in 0..n {
            let a = start + i as f64 * interval;
            self.four_bar_angle(a).unwrap();
            path[i] = self.joints[4];
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

    /// Apply the formulas.
    fn apply(&mut self) {
        for f in self.formulas.iter_mut() {
            f.apply(&mut self.joints);
        }
    }

    /// Get the length of the joints.
    pub fn joint_len(&self) -> usize {
        self.joints.len()
    }
}

/// A point-like memory layout to achieve zero copy.
pub trait Point: Sized {
    fn new(x: f64, y: f64) -> Self;
    fn x(&self) -> f64;
    fn y(&self) -> f64;

    fn pla(&self, d0: f64, a0: f64) -> Self {
        Self::new(self.x() + d0 * a0.cos(), self.y() + d0 * a0.sin())
    }

    fn plap(&self, d0: f64, a0: f64, rhs: &Self) -> Self {
        let a1 = f64::atan2(rhs.y() - self.y(), rhs.x() - self.x()) + a0;
        Self::new(self.x() + d0 * a1.cos(), self.y() + d0 * a1.sin())
    }

    fn pllp(&self, d0: f64, d1: f64, rhs: &Self, inv: bool) -> Self {
        let dx = rhs.x() - self.x();
        let dy = rhs.y() - self.y();
        let d = (self.x() - rhs.x()).hypot(self.y() - rhs.y());
        if d > d0 + d1 || d < (d0 - d1).abs() || (d < 1e-20 && d0 - d1 < 1e-20) {
            return Self::new(f64::NAN, f64::NAN);
        }
        let a = (d0 * d0 - d1 * d1 + d * d) / (2. * d);
        let h = (d0 * d0 - a * a).sqrt();
        let xm = self.x() + a * dx / d;
        let ym = self.y() + a * dy / d;
        if inv {
            Self::new(xm + h * dy / d, ym - h * dx / d)
        } else {
            Self::new(xm - h * dy / d, ym + h * dx / d)
        }
    }
}

impl Point for [f64; 2] {
    #[inline(always)]
    fn new(x: f64, y: f64) -> Self {
        [x, y]
    }
    #[inline(always)]
    fn x(&self) -> f64 {
        self[0]
    }
    #[inline(always)]
    fn y(&self) -> f64 {
        self[1]
    }
}

impl Point for (f64, f64) {
    #[inline(always)]
    fn new(x: f64, y: f64) -> Self {
        (x, y)
    }
    #[inline(always)]
    fn x(&self) -> f64 {
        self.0
    }
    #[inline(always)]
    fn y(&self) -> f64 {
        self.1
    }
}
