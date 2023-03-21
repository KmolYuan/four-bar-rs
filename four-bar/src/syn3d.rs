//! The synthesis implementation of spherical four-bar linkage mechanisms.
use std::f64::consts::{FRAC_PI_8, PI};

/// The minimum input angle bound. (π/16)
pub const MIN_ANGLE: f64 = FRAC_PI_8 * 0.5;
/// Boundary of the objective variables.
pub const BOUND: [[f64; 2]; 8] = [
    [0., PI],
    [0., PI],
    [0., PI],
    [0., PI],
    [0., PI],
    [0., PI],
    [0., PI],
    [0., PI],
];
