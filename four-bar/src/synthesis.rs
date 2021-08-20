use crate::Mechanism;
use efd::{calculate_efd, locus, normalize_efd};
pub use metaheuristics_nature::*;
use ndarray::{arr2, concatenate, Array2, AsArray, Axis, Ix2};
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
    ub: [f64; 5],
    lb: [f64; 5],
    // Normalized information
    rot: f64,
    scale: f64,
    locus: (f64, f64),
}

impl Planar {
    /// Create a new task.
    pub fn new(curve: &[[f64; 2]], n: usize, harmonic: usize) -> Self {
        let mut curve = arr2(curve);
        let end = curve.nrows() - 1;
        if (curve[[0, 0]] - curve[[end, 0]]).abs() > 1e-20
            || (curve[[0, 1]] - curve[[end, 1]]).abs() > 1e-20
        {
            // Close loop
            curve = concatenate!(Axis(0), curve, arr2(&[[curve[[0, 0]], curve[[0, 1]]]]));
        }
        let coeffs = calculate_efd(&curve, harmonic);
        let (target, rot, _, scale) = normalize_efd(&coeffs, true);
        let locus = locus(&curve);
        let mut ub = [10.; 5];
        // gamma
        ub[4] = TAU;
        let mut lb = [1e-6; 5];
        lb[4] = 0.;
        Self {
            target,
            n,
            harmonic,
            ub,
            lb,
            rot,
            scale,
            locus,
        }
    }
}

impl ObjFunc for Planar {
    type Result = Mechanism;

    fn fitness<'a, A>(&self, v: A, _: &Report) -> f64
    where
        A: AsArray<'a, f64>,
    {
        let v = v.into();
        let mut f = Mechanism::four_bar((0., 0., 0.), v[0], 1., v[1], v[2], v[3], v[4]);
        let c = arr2(&f.four_bar_loop(0., self.n));
        if path_is_nan(&c) {
            return 1e20;
        }
        let curve = concatenate!(Axis(0), c, arr2(&[[c[[0, 0]], c[[0, 1]]]]));
        let coeffs = calculate_efd(&curve, self.harmonic);
        let (coeffs, _, _, _) = normalize_efd(&coeffs, true);
        (coeffs - &self.target).mapv(f64::abs).sum()
    }

    fn result<'a, A>(&self, v: A) -> Self::Result
    where
        A: AsArray<'a, f64>,
    {
        let v = v.into();
        let c = arr2(
            &Mechanism::four_bar((0., 0., 0.), v[0], 1., v[1], v[2], v[3], v[4])
                .four_bar_loop(0., self.n),
        );
        let curve = concatenate!(Axis(0), c, arr2(&[[c[[0, 0]], c[[0, 1]]]]));
        let coeffs = calculate_efd(&curve, self.harmonic);
        let (_, rot, _, scale) = normalize_efd(&coeffs, true);
        let locus = locus(&curve);
        let rot = rot - self.rot;
        let scale = self.scale / scale;
        let locus_rot = locus.1.atan2(locus.0) + rot;
        let d = locus.1.hypot(locus.0) * scale;
        Mechanism::four_bar(
            (
                self.locus.0 - d * locus_rot.cos(),
                self.locus.1 - d * locus_rot.sin(),
                rot,
            ),
            v[0] * scale,
            scale,
            v[1] * scale,
            v[2] * scale,
            v[3] * scale,
            v[4],
        )
    }

    fn ub(&self) -> &[f64] {
        &self.ub
    }

    fn lb(&self) -> &[f64] {
        &self.lb
    }
}

/// Dimensional synthesis with default options.
pub fn synthesis(
    curve: &[[f64; 2]],
    gen: u32,
    pop: usize,
    callback: impl FnMut(Report) -> bool,
) -> (Mechanism, Vec<Report>) {
    let planar = Planar::new(curve, 720, 360);
    let de = Solver::solve(
        planar,
        De::default().task(Task::MaxGen(gen)).rpt(1).pop_num(pop),
        callback,
    );
    (de.result(), de.history())
}
