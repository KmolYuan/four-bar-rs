//! Curve (trajectory) operation functions.
//!
//! The input curve can be both a owned type `Vec<[f64; 2]>` or a pointer type
//! `&[[f64; 2]]` since the generic are copy-on-write (COW) compatible.
pub use efd::{curve_diff, Curve};

/// Remove the last point.
///
/// This function allows empty curve.
pub fn remove_last<A, C>(curve: C) -> Vec<A>
where
    A: Clone,
    C: Curve<A>,
{
    let mut curve = curve.to_curve();
    curve.pop();
    curve
}
