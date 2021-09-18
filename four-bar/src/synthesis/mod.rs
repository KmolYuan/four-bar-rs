//! The synthesis implementation of planar four-bar linkage mechanisms.
use self::guide::guide;
use crate::{FourBar, Mechanism};
use efd::{calculate_efd, locus, normalize_efd};
pub use metaheuristics_nature::*;
use ndarray::{arr2, concatenate, Array2, AsArray, Axis, Ix2};
use std::f64::consts::TAU;

mod guide;

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

// Normalization information
#[derive(Default)]
struct Norm {
    target: Array2<f64>,
    rot: f64,
    scale: f64,
    locus: (f64, f64),
}

/// Synthesis task of planar four-bar linkage.
pub struct Planar {
    norm: Norm,
    n: usize,
    harmonic: usize,
    open: bool,
    ub: Vec<f64>,
    lb: Vec<f64>,
}

impl Planar {
    /// Create a new task.
    pub fn new(curve: &[[f64; 2]], n: usize, harmonic: usize, open: bool) -> Self {
        let mut curve = arr2(curve);
        let end = curve.nrows() - 1;
        let mut ub = vec![10.; 5];
        let mut lb = vec![1e-6; 5];
        // gamma
        ub[4] = TAU;
        lb[4] = 0.;
        let norm = if open {
            // Curve guiding
            let max_d = curve
                .axis_iter(Axis(0))
                .fold([-f64::INFINITY; 2], |a, b| [a[0].max(b[0]), a[1].max(b[1])]);
            let min_d = curve
                .axis_iter(Axis(0))
                .fold([f64::INFINITY; 2], |a, b| [a[0].min(b[0]), a[1].min(b[1])]);
            let max_d = (max_d[0] - min_d[0]).max(max_d[1] - min_d[1]);
            // Open path guiding points
            ub.push(max_d);
            lb.push(1e-6);
            for _ in 0..3 {
                ub.push(max_d);
                lb.push(1e-6);
                ub.push(TAU);
                lb.push(0.);
            }
            ub.push(max_d);
            lb.push(1e-6);
            Norm {
                target: curve,
                ..Default::default()
            }
        } else {
            // Close loop
            if (curve[[0, 0]] - curve[[end, 0]]).abs() > 1e-20
                || (curve[[0, 1]] - curve[[end, 1]]).abs() > 1e-20
            {
                curve = concatenate!(Axis(0), curve, arr2(&[[curve[[0, 0]], curve[[0, 1]]]]));
            }
            let coeffs = calculate_efd(&curve, harmonic);
            let (target, rot, _, scale) = normalize_efd(&coeffs, true);
            Norm {
                target,
                rot,
                scale,
                locus: locus(&curve),
            }
        };
        Self {
            norm,
            n,
            harmonic,
            open,
            ub,
            lb,
        }
    }

    fn four_bar(v: &[f64]) -> FourBar {
        FourBar {
            p0: (0., 0.),
            a: 0.,
            l0: v[0],
            l1: 1.,
            l2: v[1],
            l3: v[2],
            l4: v[3],
            g: v[4],
        }
    }
}

impl ObjFunc for Planar {
    type Result = FourBar;

    fn fitness(&self, v: &[f64], _: &Report) -> f64 {
        let mut f = Mechanism::four_bar(Self::four_bar(v));
        let c = arr2(&f.four_bar_loop(0., self.n));
        if path_is_nan(&c) {
            return 1e20;
        }
        let curve = concatenate!(Axis(0), c, arr2(&[[c[[0, 0]], c[[0, 1]]]]));
        let coeffs = calculate_efd(&curve, self.harmonic);
        let (coeffs, ..) = normalize_efd(&coeffs, true);
        if self.open {
            let mut c = self.norm.target.clone();
            guide(&mut c, &v[5..]);
            let curve = concatenate!(Axis(0), c, arr2(&[[c[[0, 0]], c[[0, 1]]]]));
            let coeffs = calculate_efd(&curve, self.harmonic);
            let (target, ..) = normalize_efd(&coeffs, true);
            (coeffs - &target).mapv(f64::abs).sum()
        } else {
            (coeffs - &self.norm.target).mapv(f64::abs).sum()
        }
    }

    fn result(&self, v: &[f64]) -> Self::Result {
        let c = arr2(&Mechanism::four_bar(Self::four_bar(v)).four_bar_loop(0., self.n));
        let curve = concatenate!(Axis(0), c, arr2(&[[c[[0, 0]], c[[0, 1]]]]));
        let coeffs = calculate_efd(&curve, self.harmonic);
        let (_, rot, _, scale) = normalize_efd(&coeffs, true);
        let (norm_rot, norm_scale, norm_locus) = if self.open {
            let mut c = self.norm.target.clone();
            guide(&mut c, &v[5..]);
            let curve = concatenate!(Axis(0), c, arr2(&[[c[[0, 0]], c[[0, 1]]]]));
            let coeffs = calculate_efd(&curve, self.harmonic);
            let (_, rot, _, scale) = normalize_efd(&coeffs, true);
            (rot, scale, locus(&curve))
        } else {
            (self.norm.rot, self.norm.scale, self.norm.locus)
        };
        let locus = locus(&curve);
        let rot = rot - norm_rot;
        let scale = norm_scale / scale;
        let locus_rot = locus.1.atan2(locus.0) + rot;
        let d = locus.1.hypot(locus.0) * scale;
        FourBar {
            p0: (
                norm_locus.0 - d * locus_rot.cos(),
                norm_locus.1 - d * locus_rot.sin(),
            ),
            a: rot,
            l0: v[0] * scale,
            l1: scale,
            l2: v[1] * scale,
            l3: v[2] * scale,
            l4: v[3] * scale,
            g: v[4],
        }
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
    open: bool,
    callback: impl FnMut(&Report) -> bool,
) -> (FourBar, Vec<Report>) {
    let planar = Planar::new(curve, 720, 360, open);
    let s = Solver::solve(
        planar,
        De::default().task(Task::MaxGen(gen)).rpt(1).pop_num(pop),
        callback,
    );
    (s.result(), s.reports())
}
