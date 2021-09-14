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

impl From<[f64; 2]> for Point {
    fn from(c: [f64; 2]) -> Self {
        Self(c[0], c[1])
    }
}

pub(crate) fn guide(c: &mut Array2<f64>, v: &[f64]) {
    let last = [c[[c.nrows() - 1, 0]], c[[c.nrows() - 1, 1]]];
    let dx = last[0] - c[[c.nrows() - 2, 0]];
    let dy = last[1] - c[[c.nrows() - 2, 1]];
    let a = dy.atan2(dx);
    let mut pts = vec![last, [last[0] + v[0] * a.cos(), last[1] + v[0] * a.sin()]];
    for i in (1..v.len() - 1).step_by(2) {
        let end = pts[pts.len() - 1];
        pts.push([
            end[0] + v[i] * v[i + 1].cos(),
            end[1] + v[i] * v[i + 1].sin(),
        ]);
    }
    let first = [c[[0, 0]], c[[0, 1]]];
    let dx = first[0] - c[[1, 0]];
    let dy = first[1] - c[[1, 1]];
    let a = dy.atan2(dx);
    pts.push([
        first[0] + v[v.len() - 1] * a.cos(),
        first[1] + v[v.len() - 1] * a.sin(),
    ]);
    pts.push(first);
    let bs = BSpline::new(
        pts.len(),
        pts.iter().map(|c| Point::from(c.clone())).collect(),
        Array1::linspace(0., 1., pts.len() * 2 + 1).to_vec(),
    );
    let mut new_curve = Vec::new();
    for f in Array1::linspace(0.1, 0.9, 8).to_vec() {
        let p = bs.point(f);
        new_curve.push([p.0, p.1])
    }
    *c = concatenate![Axis(0), *c, arr2(&new_curve)];
}
