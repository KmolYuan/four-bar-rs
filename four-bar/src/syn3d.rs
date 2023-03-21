//! The synthesis implementation of spherical four-bar linkage mechanisms.
pub use crate::syn2d::{Mode, MIN_ANGLE};
use std::f64::consts::PI;

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

/// TODO: Path generation task of spherical four-bar linkage.
#[allow(unused)]
pub struct SphericalSyn {
    /// Target coefficient
    pub efd: efd::Efd3,
    mode: Mode,
    // How many points need to be generated or compared
    res: usize,
}
