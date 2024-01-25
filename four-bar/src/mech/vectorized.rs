//! Vectorized number support.
use crate::{mech::Stat, *};
use std::f64::consts::{PI, TAU};

/// Support a type transforming from a vectored number.
pub trait FromVectorized<const N: usize>: Sized {
    /// Lower & upper bounds
    const BOUND: [[f64; 2]; N];
    /// Lower & upper bounds for partial synthesis
    const BOUND_PARTIAL: &'static [[f64; 2]];

    /// Create a new instance from a vector.
    fn from_vectorized(v: [f64; N], stat: mech::Stat) -> Self;

    /// Create a new instance from a vector with `C1B1` stat.
    fn from_vectorized_s1(v: [f64; N]) -> Self {
        Self::from_vectorized(v, mech::Stat::C1B1)
    }
}

/// Support a type transforming to a vectored number.
pub trait IntoVectorized {
    /// Convert the type to a vector.
    fn into_vectorized(self) -> (Vec<f64>, mech::Stat);
}

// Concat const slices by their variable names, currently only support
// non-generic slices.
const fn concat_slices<T: Copy, const N1: usize, const N2: usize, const N3: usize>(
    a: [T; N1],
    b: [T; N2],
) -> [T; N3]
where
    efd::na::Const<N1>: efd::na::DimNameAdd<efd::na::Const<N2>, Output = efd::na::Const<N3>>,
{
    let mut out = [a[0]; N3];
    let mut i = 0;
    while i < N1 {
        out[i] = a[i];
        i += 1;
    }
    let mut j = 0;
    while j < N2 {
        out[i] = b[j];
        i += 1;
        j += 1;
    }
    out
}

impl FromVectorized<5> for NormFourBar {
    const BOUND: [[f64; 2]; 5] = {
        const K: f64 = 6.;
        concat_slices([[1. / K, K]; 4], [[0., TAU]; 1])
    };
    const BOUND_PARTIAL: &'static [[f64; 2]] = &concat_slices(Self::BOUND, [[0., TAU]; 2]);

    fn from_vectorized(v: [f64; 5], stat: Stat) -> Self {
        let [l1, l3, l4, l5, g] = v;
        Self { l1, l3, l4, l5, g, stat: stat as Stat }
    }
}

impl IntoVectorized for NormFourBar {
    fn into_vectorized(self) -> (Vec<f64>, Stat) {
        let code = vec![self.l1, self.l3, self.l4, self.l5, self.g];
        (code, self.stat)
    }
}

impl FromVectorized<6> for MNormFourBar {
    const BOUND: [[f64; 2]; 6] = {
        const K: f64 = 6.;
        concat_slices([[1. / K, K]; 4], [[0., TAU]; 2])
    };
    const BOUND_PARTIAL: &'static [[f64; 2]] = &concat_slices(Self::BOUND, [[0., TAU]; 2]);

    fn from_vectorized(v: [f64; 6], stat: Stat) -> Self {
        let [l1, l3, l4, l5, g, e] = v;
        Self { base: NormFourBar { l1, l3, l4, l5, g, stat }, e }
    }
}

impl IntoVectorized for MNormFourBar {
    fn into_vectorized(self) -> (Vec<f64>, Stat) {
        let code = vec![self.l1, self.l3, self.l4, self.l5, self.g, self.e];
        (code, self.stat)
    }
}

impl FromVectorized<6> for SNormFourBar {
    const BOUND: [[f64; 2]; 6] = concat_slices([[1e-4, PI]; 5], [[0., PI]; 1]);
    const BOUND_PARTIAL: &'static [[f64; 2]] = &concat_slices(Self::BOUND, [[0., TAU]; 2]);

    fn from_vectorized(v: [f64; 6], stat: Stat) -> Self {
        let [l1, l2, l3, l4, l5, g] = v;
        Self { l1, l2, l3, l4, l5, g, stat }
    }
}

impl IntoVectorized for SNormFourBar {
    fn into_vectorized(self) -> (Vec<f64>, Stat) {
        let Self { l1, l2, l3, l4, l5, g, stat } = self;
        (vec![l1, l2, l3, l4, l5, g], stat)
    }
}
