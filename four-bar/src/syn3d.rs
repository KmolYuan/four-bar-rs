//! The synthesis implementation of spherical four-bar linkage mechanisms.
pub use crate::syn2d::{Mode, MIN_ANGLE};
use crate::{efd::Curve, *};
use std::f64::consts::{PI, TAU};

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
const INFEASIBLE: (f64, SFourBar) = (1e10, SFourBar::ZERO);

syn2d::impl_obj! {
    /// Path generation task of spherical four-bar linkage.
    struct SphericalSyn, Efd3, SFourBar, SNormFourBar, [f64; 3], 6
}
