//! Vectorized number support.
use crate::*;
use efd::na;

/// Support a type transforming from a vectored number.
pub trait FromVectorized: Sized {
    /// Dimension of the type.
    type Dim: na::DimName;

    /// Create a new instance from a vector.
    fn from_vectorized(v: &[f64], stat: fb::Stat) -> Result<Self, std::array::TryFromSliceError>;

    /// Create a new instance from a vector with `C1B1` stat.
    fn from_vectorized_s1(v: &[f64]) -> Result<Self, std::array::TryFromSliceError> {
        Self::from_vectorized(v, fb::Stat::C1B1)
    }
}

/// Support a type transforming to a vectored number.
pub trait IntoVectorized {
    /// Convert the type to a vector.
    fn into_vectorized(self) -> (Vec<f64>, fb::Stat);
}
