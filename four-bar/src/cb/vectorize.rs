use crate::*;

macro_rules! impl_vec {
    ($($ty:ty{$num:literal}),+) => {$(
        impl Vectorize<$num> for $ty {
            fn from_array(v: [f64; $num], inv: bool) -> Self {
                Self::new(v, inv)
            }

            fn to_array(self) -> (bool, [f64; $num]) {
                (self.inv(), self.as_array())
            }
        }
    )+};
}

/// Transform data to static array and back.
pub trait Vectorize<const N: usize>: Sized {
    /// Length of the array.
    const N: usize = N;

    /// Convert from array.
    fn from_array(v: [f64; N], inv: bool) -> Self;
    /// Convert to array.
    fn to_array(self) -> (bool, [f64; N]);

    /// Convert from unknown-size slice.
    fn from_slice(v: &[f64], inv: bool) -> Result<Self, std::array::TryFromSliceError> {
        Ok(Self::from_array(v.try_into()?, inv))
    }
}

impl_vec!(NormFourBar{5}, FourBar{9}, SNormFourBar{6}, SFourBar{13});
