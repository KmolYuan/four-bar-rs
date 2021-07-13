/// A point-like memory layout to achieve zero copy.
pub trait Point: Sized {
    fn point(x: f64, y: f64) -> Self;
    fn x(&self) -> f64;
    fn y(&self) -> f64;

    fn pla(&self, d0: f64, a0: f64) -> Self {
        Self::point(self.x() + d0 * a0.cos(), self.y() + d0 * a0.sin())
    }

    fn plap(&self, d0: f64, a0: f64, rhs: &Self) -> Self {
        let a1 = f64::atan2(rhs.y() - self.y(), rhs.x() - self.x()) + a0;
        Self::point(self.x() + d0 * a1.cos(), self.y() + d0 * a1.sin())
    }

    fn pllp(&self, d0: f64, d1: f64, rhs: &Self, inv: bool) -> Self {
        let dx = rhs.x() - self.x();
        let dy = rhs.y() - self.y();
        let d = dx.hypot(dy);
        if d > d0 + d1 || d < (d0 - d1).abs() || (d < 1e-20 && d0 - d1 < 1e-20) {
            return Self::point(f64::NAN, f64::NAN);
        }
        let a = (d0 * d0 - d1 * d1 + d * d) / (2. * d);
        let h = (d0 * d0 - a * a).sqrt();
        let xm = self.x() + a * dx / d;
        let ym = self.y() + a * dy / d;
        if inv {
            Self::point(xm + h * dy / d, ym - h * dx / d)
        } else {
            Self::point(xm - h * dy / d, ym + h * dx / d)
        }
    }

    fn ppp(&self, c2: &Self, c3: &Self) -> Self {
        let dx = c2.x() - self.x();
        let dy = c2.y() - self.y();
        let d = dx.hypot(dy);
        let a = dy.atan2(dx);
        Self::point(c3.x() + d * a.cos(), c3.y() + d * a.sin())
    }
}

impl Point for [f64; 2] {
    #[inline(always)]
    fn point(x: f64, y: f64) -> Self {
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
    fn point(x: f64, y: f64) -> Self {
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
