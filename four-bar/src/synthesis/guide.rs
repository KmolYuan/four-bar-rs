use bspline::BSpline;
use ndarray::{arr2, concatenate, Array1, Array2, Axis};
use std::ops::{Add, Mul};

#[derive(Copy, Clone)]
struct Point(f64, f64);

impl Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Mul<f64> for Point {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

pub(crate) fn guide(c: &mut Array2<f64>, v: &[f64]) {
    let last = [c[[c.nrows() - 1, 0]], c[[c.nrows() - 1, 1]]];
    let dx = last[0] - c[[c.nrows() - 2, 0]];
    let dy = last[1] - c[[c.nrows() - 2, 1]];
    let mut a = dy.atan2(dx);
    let mut guide = vec![last, [last[0] + v[0] * a.cos(), last[1] + v[0] * a.sin()]];
    for i in (1..v.len() - 1).step_by(2) {
        let end = guide[guide.len() - 1];
        a += v[i + 1];
        guide.push([end[0] + v[i] * a.cos(), end[1] + v[i] * a.sin()]);
    }
    let first = [c[[0, 0]], c[[0, 1]]];
    let dx = first[0] - c[[1, 0]];
    let dy = first[1] - c[[1, 1]];
    let a = dy.atan2(dx);
    guide.push([
        first[0] + v[v.len() - 1] * a.cos(),
        first[1] + v[v.len() - 1] * a.sin(),
    ]);
    guide.push(first);
    // TODO: knots
    let bs = BSpline::new(
        4,
        guide.iter().map(|c| Point(c[0], c[1])).collect(),
        vec![0., 0., 0., 0., 0., 1., 2., 3., 4., 4., 4., 4., 4.],
    );
    let domain = bs.knot_domain();
    let mut new_curve = Vec::new();
    for f in Array1::linspace(domain.0, domain.1, 20).to_vec() {
        let p = bs.point(f);
        new_curve.push([p.0, p.1])
    }
    *c = concatenate![Axis(0), *c, arr2(&new_curve)];
}
