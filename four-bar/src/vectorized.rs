//! Vectorized number support.
//!
//! + `FromVectorized`: Support a type transforming from a vectored number.
//! + `IntoVectorized`: Support a type transforming to a vectored number.
use crate::*;
use efd::na;

/// Support a type transforming from a vectored number.
pub trait FromVectorized: Sized {
    /// Dimension of the type.
    type Dim: na::DimName;

    /// Create a new instance from a vector.
    fn from_vectorized(v: &[f64], stat: u8) -> Result<Self, std::array::TryFromSliceError>;
}

/// Support a type transforming to a vectored number.
pub trait IntoVectorized {
    /// Convert the type to a vector.
    fn into_vectorized(self) -> (Vec<f64>, u8);
}
