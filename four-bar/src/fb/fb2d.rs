use super::*;
use std::f64::consts::FRAC_PI_6;

/// Normalized four-bar linkage.
///
/// + Buffer order: `[l1, l3, l4, l5, g]`
///
/// # Parameters
///
/// + Ground link `l1`
/// + Driver link `l2=1`
/// + Coupler link `l3`
/// + Follower link `l4`
/// + Extanded link `l5`
/// + Coupler link angle `g`
pub type NormFourBar = NormFourBarBase<[f64; 5]>;
/// Four-bar linkage with offset.
///
/// + Buffer 1 order: `[p0x, p0y, a, l2]`
/// + Buffer 2 order: `[l1, l3, l4, l5, g]`
///
/// # Parameters
///
/// + X offset `p0x`
/// + Y offset `p0y`
/// + Angle offset `a`
/// + Ground link `l1`
/// + Driver link `l2`
/// + Coupler link `l3`
/// + Follower link `l4`
/// + Extanded link `l5`
/// + Coupler link angle `g`
pub type FourBar = FourBarBase<[f64; 4], [f64; 5]>;

impl Normalized<efd::D2> for NormFourBar {
    type De = FourBar;

    fn denormalize(&self) -> Self::De {
        FourBar { buf: [0., 0., 0., self.l2()], norm: self.clone() }
    }

    fn normalize(de: &Self::De) -> Self {
        let l2 = de.l2();
        let mut norm = de.norm.clone();
        norm.buf[..4].iter_mut().for_each(|x| *x /= l2);
        norm
    }
}

impl NormFourBar {
    impl_parm_method! {
        /// X offset of the driver link pivot.
        fn p0x(self) -> f64 { 0. }
        /// Y offset of the driver link pivot.
        fn p0y(self) -> f64 { 0. }
        /// Angle offset of the ground link.
        fn a(self) -> f64 { 0. }
        /// Length of the ground link.
        fn l1, l1_mut(self) -> f64 { self.buf[0] }
        /// Length of the driver link.
        fn l2(self) -> f64 { 1. }
        /// Length of the coupler link.
        fn l3, l3_mut(self) -> f64 { self.buf[1] }
        /// Length of the follower link.
        fn l4, l4_mut(self) -> f64 { self.buf[2] }
        /// Length of the extended link.
        fn l5, l5_mut(self) -> f64 { self.buf[3] }
        /// Angle of the extended link on the coupler.
        fn g, g_mut(self) -> f64 { self.buf[4] }
        /// Inverse coupler and follower to another circuit.
        fn inv, inv_mut(self) -> bool { self.inv }
    }

    /// Return the type of this linkage.
    pub fn ty(&self) -> FourBarTy {
        FourBarTy::from_loop([self.l1(), self.l2(), self.l3(), self.l4()])
    }
}

impl FourBar {
    /// An example crank rocker.
    pub const fn example() -> Self {
        Self::new([0., 0., 0., 35.], [90., 70., 70., 45., FRAC_PI_6], false)
    }

    impl_parm_method! {
        /// X offset of the driver link pivot.
        fn p0x, p0x_mut(self) -> f64 { self.buf[0] }
        /// Y offset of the driver link pivot.
        fn p0y, p0y_mut(self) -> f64 { self.buf[1] }
        /// Angle offset of the ground link.
        fn a, a_mut(self) -> f64 { self.buf[2] }
        /// Length of the ground link.
        fn l1, l1_mut(self) -> f64 { self.norm.buf[0] }
        /// Length of the driver link.
        fn l2, l2_mut(self) -> f64 { self.buf[3] }
        /// Length of the coupler link.
        fn l3, l3_mut(self) -> f64 { self.norm.buf[1] }
        /// Length of the follower link.
        fn l4, l4_mut(self) -> f64 { self.norm.buf[2] }
        /// Length of the extended link.
        fn l5, l5_mut(self) -> f64 { self.norm.buf[3] }
        /// Angle of the extended link on the coupler.
        fn g, g_mut(self) -> f64 { self.norm.buf[4] }
        /// Inverse coupler and follower to another circuit.
        fn inv, inv_mut(self) -> bool { self.norm.inv }
    }

    /// Return the type of this linkage.
    pub fn ty(&self) -> FourBarTy {
        FourBarTy::from_loop([self.l1(), self.l2(), self.l3(), self.l4()])
    }
}

impl Transformable<efd::D2> for FourBar {
    fn transform_inplace(&mut self, trans: &efd::Transform<<efd::D2 as EfdDim>::Trans>) {
        let [p0x, p0y] = trans.trans();
        *self.p0x_mut() += p0x;
        *self.p0y_mut() += p0y;
        *self.a_mut() += trans.rot().angle();
        let scale = trans.scale();
        *self.l2_mut() *= scale;
        self.norm.buf[..4].iter_mut().for_each(|x| *x *= scale);
    }
}

impl CurveGen<efd::D2> for FourBar {
    fn is_valid(&self) -> bool {
        let mut v = [self.l1(), self.l2(), self.l3(), self.l4()];
        v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        v[3] < v[..3].iter().sum()
    }

    fn pos(&self, t: f64) -> Option<[efd::Coord<efd::D2>; 5]> {
        curve_interval(self, t)
    }

    fn angle_bound(&self) -> Option<[f64; 2]> {
        self.is_valid()
            .then(|| angle_bound([self.l1(), self.l2(), self.l3(), self.l4()]))
    }
}

fn angle([x, y]: [f64; 2], d: f64, a: f64) -> [f64; 2] {
    [x + d * a.cos(), y + d * a.sin()]
}

fn angle_with([x1, y1]: [f64; 2], [x2, y2]: [f64; 2], d: f64, a: f64) -> [f64; 2] {
    let a = (y2 - y1).atan2(x2 - x1) + a;
    [x1 + d * a.cos(), y1 + d * a.sin()]
}

fn circle2([x1, y1]: [f64; 2], [x2, y2]: [f64; 2], r1: f64, r2: f64, inv: bool) -> [f64; 2] {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let r = dx.hypot(dy);
    if r > r1 + r2 || r < (r1 - r2).abs() || (r < f64::EPSILON && (r1 - r2).abs() < f64::EPSILON) {
        return [f64::NAN; 2];
    }
    let a = 0.5 * (r1 * r1 - r2 * r2 + r * r) / r;
    let h = (r1 * r1 - a * a).sqrt();
    let c = dx / r;
    let s = dy / r;
    let xm = x1 + a * c;
    let ym = y1 + a * s;
    if inv {
        [xm + h * s, ym - h * c]
    } else {
        [xm - h * s, ym + h * c]
    }
}

fn curve_interval(fb: &FourBar, b: f64) -> Option<[[f64; 2]; 5]> {
    let [p0x, p0y, a, l2] = fb.buf;
    let NormFourBar { buf: [l1, l3, l4, l5, g], inv } = fb.norm;
    let p0 = [p0x, p0y];
    let p1 = angle(p0, l1, a);
    let p2 = angle(p0, l2, a + b);
    let p3 = if (l1 - l3).abs() < f64::EPSILON && (l2 - l4).abs() < f64::EPSILON {
        // Special case
        let [p0x, p0y] = p0;
        let [p1x, p1y] = p1;
        let [p2x, p2y] = p2;
        let dx = p2x - p0x;
        let dy = p2y - p0y;
        let d = dx.hypot(dy);
        let a = dy.atan2(dx);
        [p1x + d * a.cos(), p1y + d * a.sin()]
    } else {
        circle2(p2, p1, l3, l4, inv)
    };
    let p4 = angle_with(p2, p3, l5, g);
    let js = [p0, p1, p2, p3, p4];
    js.iter().flatten().all(|x| x.is_finite()).then_some(js)
}
