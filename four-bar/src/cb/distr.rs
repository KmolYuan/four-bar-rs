use crate::*;
use mh::rand::{Distribution, Rng};

/// Uniform distribution of the [`NormFourBarBase`] type.
pub struct NormFbDistr<const N: usize>;

type NormFb<const N: usize> = NormFourBarBase<[f64; N]>;

impl<const N: usize> Distribution<[NormFb<N>; 2]> for NormFbDistr<N>
where
    NormFb<N>: syn::SynBound,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [NormFb<N>; 2] {
        let v = <NormFb<N> as syn::SynBound>::BOUND[..<NormFb<N> as syn::SynBound>::BOUND_NUM]
            .iter()
            .map(|&[u, l]| rng.gen_range(u..l))
            .collect::<Vec<_>>();
        [false, true].map(|inv| NormFb::<N>::try_from(v.as_slice()).unwrap().with_inv(inv))
    }
}

/// Implement this trait to support codebook functions.
pub trait Code<D: efd::EfdDim, const N: usize>: Normalized<D> + CurveGen<D> + Sized {
    /// Random distribution
    type Distr: Distribution<[Self; 2]> + Sync;

    /// Return the distribution.
    fn distr() -> Self::Distr;
    /// Create entities from code.
    fn from_code(code: [f64; N], inv: bool) -> Self;
    /// Convert entities to code.
    fn to_code(self) -> ([f64; N], bool);

    /// Generate curve and check the curve type.
    fn get_curve(&self, res: usize, is_open: bool) -> Option<Vec<efd::Coord<D>>> {
        self.angle_bound()
            .check_mode(is_open)
            .to_value()
            .map(|[t1, t2]| self.curve_in(t1, t2, res))
    }
}

impl<D: efd::EfdDim, const N: usize> Code<D, N> for NormFb<N>
where
    Self: Normalized<D> + CurveGen<D> + syn::SynBound,
{
    type Distr = NormFbDistr<N>;

    fn distr() -> Self::Distr {
        NormFbDistr
    }

    fn from_code(code: [f64; N], inv: bool) -> Self {
        Self::new(code, inv)
    }

    fn to_code(self) -> ([f64; N], bool) {
        (self.buf, self.inv)
    }
}
