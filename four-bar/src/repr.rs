//! Provide vectorized conversion for normalized four-bar linkages.
use crate::FourBar;

/// Grashof transform for any non-Grashof linkages (in vector form).
pub fn grashof_transform(v: &[f64]) -> [f64; 5] {
    let mut four = [v[0], 1., v[1], v[2]];
    four.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    if four[0] + four[3] - four[1] - four[2] < 0. && (four[0] == 1. || four[0] == v[0]) {
        [v[0], v[1], v[2], v[3], v[4]]
    } else {
        let l1 = four[0];
        [four[1] / l1, four[3] / l1, four[2] / l1, v[3] / l1, v[4]]
    }
}

/// Create a normalized four-bar linkage from a vector.
pub const fn four_bar_v([l0, l2, l3, l4, g]: [f64; 5], inv: bool) -> FourBar {
    FourBar {
        p0: [0., 0.],
        a: 0.,
        l0,
        l1: 1.,
        l2,
        l3,
        l4,
        g,
        inv,
    }
}
