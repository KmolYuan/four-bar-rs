use crate::Mechanism;
use efd::{calculate_efd, inverse_transform, locus, normalize_efd, rotate_contour};
pub use metaheuristics_nature::*;
use ndarray::{arr2, concatenate, Array1, Array2, ArrayView1, AsArray, Axis, Ix2};
use std::f64::consts::TAU;

fn path_is_nan<'a, V>(path: V) -> bool
where
    V: AsArray<'a, f64, Ix2>,
{
    let path = path.into();
    for i in 0..path.nrows() {
        if path[[i, 0]].is_nan() || path[[i, 1]].is_nan() {
            return true;
        }
    }
    false
}

/// Synthesis task of planar four-bar linkage.
pub struct Planar {
    /// The target coefficients.
    pub target: Array2<f64>,
    n: usize,
    harmonic: usize,
    ub: Array1<f64>,
    lb: Array1<f64>,
    // Normalized information
    rot: f64,
    locus: (f64, f64),
}

impl Planar {
    /// Create a new task.
    pub fn new<'a, V>(curve: V, n: usize, harmonic: usize) -> Self
    where
        V: AsArray<'a, f64, Ix2>,
    {
        let mut curve = curve.into().into_owned();
        let end = curve.nrows() - 1;
        if (curve[[0, 0]] - curve[[end, 0]]).abs() > 1e-20
            || (curve[[0, 1]] - curve[[end, 1]]).abs() > 1e-20
        {
            // Close loop
            curve = concatenate!(Axis(0), curve, arr2(&[[curve[[0, 0]], curve[[0, 1]]]]));
        }
        let coeffs = calculate_efd(&curve, harmonic);
        let (target, rot) = normalize_efd(&coeffs, true);
        let locus = locus(&curve);
        let mut ub = Array1::ones(5) * 10.;
        // gamma
        ub[4] = TAU;
        let mut lb = Array1::ones(5) * 1e-6;
        lb[4] = 0.;
        Self {
            target,
            n,
            harmonic,
            ub,
            lb,
            rot,
            locus,
        }
    }
}

impl ObjFunc for Planar {
    type Result = Array2<f64>;

    fn fitness<'a, A>(&self, v: A, _: &Report) -> f64
    where
        A: AsArray<'a, f64>,
    {
        let v = v.into();
        let mut f = Mechanism::four_bar((0., 0.), 0., v[0], 1., v[1], v[2], v[3], v[4]);
        let c = arr2(&f.four_bar_loop(0., self.n));
        if path_is_nan(&c) {
            return 1e20;
        }
        let curve = concatenate!(Axis(0), c, arr2(&[[c[[0, 0]], c[[0, 1]]]]));
        let coeffs = calculate_efd(&curve, self.harmonic);
        let (coeffs, _) = normalize_efd(&coeffs, true);
        (coeffs - &self.target).mapv(f64::abs).sum()
    }

    fn result<'a, A>(&self, v: A) -> Self::Result
    where
        A: AsArray<'a, f64>,
    {
        let v = v.into();
        let mut f = Mechanism::four_bar((0., 0.), 0., v[0], 1., v[1], v[2], v[3], v[4]);
        let curve1 = arr2(&f.four_bar_loop(0., self.n));
        let coeffs = calculate_efd(&curve1, self.harmonic);
        let (coeffs, _) = normalize_efd(&coeffs, true);
        let contour = inverse_transform(&coeffs, self.locus, self.n, None);
        rotate_contour(&contour, -self.rot, self.locus)
    }

    fn ub(&self) -> ArrayView1<f64> {
        self.ub.view()
    }

    fn lb(&self) -> ArrayView1<f64> {
        self.lb.view()
    }
}

/// Dimensional synthesis with default options.
pub fn synthesis<C>(curve: &[[f64; 2]], callback: impl Callback<C>) -> Vec<[f64; 2]> {
    let planar = Planar::new(&arr2(curve), 720, 360);
    let contour = DE::solve(
        planar,
        DESetting::default().task(Task::MaxGen(40)).pop_num(400),
        callback,
    )
    .result();
    contour.axis_iter(Axis(0)).map(|v| [v[0], v[1]]).collect()
}
