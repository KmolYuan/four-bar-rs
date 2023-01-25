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

pub trait Vectorize<const N: usize>: Sized {
    const N: usize = N;

    fn from_array(v: [f64; N]) -> Self;
    fn to_array(self) -> [f64; N];

    fn from_slice(v: &[f64]) -> Result<Self, std::array::TryFromSliceError> {
        Ok(Self::from_array(v.try_into()?))
    }
}

impl_vec!(NormFourBar{5}, FourBar{9}, SNormFourBar{6}, SFourBar{13});
