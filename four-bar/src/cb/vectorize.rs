use crate::*;

macro_rules! impl_vec {
    ($($ty:ty{$num:literal}),+) => {$(
        impl Vectorize<$num> for $ty {
            fn from_array(v: [f64; $num]) -> Self {
                Self::new(v, false)
            }

            fn to_array(self) -> [f64; $num] {
                self.as_array()
            }
        }
    )+};
}

/// Transform data to static array and back.
pub trait Vectorize<const N: usize>: Sized {
    /// Length of the array.
    const N: usize = N;

    /// Convert from array.
    fn from_array(v: [f64; N]) -> Self;
    /// Convert to array.
    fn to_array(self) -> [f64; N];

    /// Convert from unknown-size slice.
    fn from_slice(v: &[f64]) -> Result<Self, std::array::TryFromSliceError> {
        Ok(Self::from_array(v.try_into()?))
    }
}

impl_vec!(NormFourBar{5}, FourBar{9}, SNormFourBar{6}, SFourBar{13});
