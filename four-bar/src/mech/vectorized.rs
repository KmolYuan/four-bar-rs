//! Vectorized number support.
use crate::*;

/// Support a type transforming from a vectored number.
pub trait FromVectorized<const N: usize>: Sized {
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
