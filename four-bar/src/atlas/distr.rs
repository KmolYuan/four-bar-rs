use crate::{mech::*, syn};
use mh::rand::{Distribution, Rng};

/// Uniform distribution for mechinism types.
pub struct Distr<M, const N: usize> {
    _marker: std::marker::PhantomData<M>,
}

impl<M, const N: usize> Distr<M, N> {
    /// Create a new instance.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { _marker: std::marker::PhantomData }
    }
}

impl<M, const N: usize> Distribution<Vec<M>> for Distr<M, N>
where
    M: syn::SynBound<N>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec<M> {
        let v = M::BOUND.map(|[u, l]| rng.gen_range(u..l));
        M::from_vectorized_s1(v).to_states()
    }
}

/// Implement this trait to support atlas functions.
pub trait Code<const N: usize, const D: usize>:
    Normalized<D> + CurveGen<D> + syn::SynBound<N> + IntoVectorized + 'static
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Create entities from code.
    fn from_code(code: &[f64], stat: u8) -> Self {
        Self::from_vectorized(code.try_into().unwrap(), Stat::try_from(stat).unwrap())
    }

    /// Convert entities to code.
    fn to_code(self) -> (Vec<f64>, u8) {
        let (code, stat) = self.into_vectorized();
        (code, stat as u8)
    }

    /// Generate curve and check the curve type.
    fn get_curve(&self, res: usize, is_open: bool) -> Option<Vec<[f64; D]>> {
        self.angle_bound()
            .check_mode(is_open)
            .to_value()
            .map(|[t1, t2]| self.curve_in(t1, t2, res))
    }
}

impl<M, const N: usize, const D: usize> Code<N, D> for M
where
    M: Normalized<D> + CurveGen<D> + syn::SynBound<N> + IntoVectorized + 'static,
    efd::U<D>: efd::EfdDim<D>,
{
}
