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

/// Geometry error between two closed curves.
///
/// The curves must have the same length.
pub fn geo_err(target: &[[f64; 2]], curve: &[[f64; 2]]) -> f64 {
    debug_assert!(!target.is_empty());
    debug_assert_eq!(target.len(), curve.len());
    // Find the starting point (correlation)
    let [tx, ty] = &target[0];
    let (i, _) = curve
        .iter()
        .map(|[x, y]| (tx - x).hypot(ty - y))
        .enumerate()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap();
    // Error
    target
        .iter()
        .zip(curve.iter().cycle().skip(i))
        .map(|([x1, y1], [x2, y2])| (x1 - x2).hypot(y1 - y2))
        .sum()
}
