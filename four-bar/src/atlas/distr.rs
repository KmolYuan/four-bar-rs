use crate::{
    fb::{CurveGen, FromVectorized, IntoVectorized, Normalized, Statable},
    syn,
};
use mh::rand::{Distribution, Rng};

/// Uniform distribution for mechinism types.
pub struct Distr<M> {
    _marker: std::marker::PhantomData<M>,
}

impl<M> Distr<M> {
    /// Create a new instance.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { _marker: std::marker::PhantomData }
    }
}

impl<M> Distribution<Vec<M>> for Distr<M>
where
    M: syn::SynBound + Statable + FromVectorized + Sync + Clone,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec<M> {
        let bound = <M as syn::SynBound>::BOUND;
        let v = bound[..bound.len() - 2]
            .iter()
            .map(|&[u, l]| rng.gen_range(u..l))
            .collect::<Vec<_>>();
        M::from_vectorized(&v, 0).unwrap().get_states()
    }
}

/// Implement this trait to support atlas functions.
pub trait Code<D: efd::EfdDim>:
    Normalized<D>
    + CurveGen<D>
    + syn::SynBound
    + Statable
    + FromVectorized
    + IntoVectorized
    + Clone
    + 'static
{
    /// The dimension of the code.
    fn dim() -> usize {
        <<Self as FromVectorized>::Dim as efd::na::DimName>::dim()
    }

    /// Create entities from code.
    fn from_code(code: &[f64], stat: u8) -> Self {
        Self::from_vectorized(code, stat).unwrap()
    }

    /// Convert entities to code.
    fn to_code(self) -> (Vec<f64>, u8) {
        self.into_vectorized()
    }

    /// Generate curve and check the curve type.
    fn get_curve(&self, res: usize, is_open: bool) -> Option<Vec<efd::Coord<D>>> {
        self.angle_bound()
            .check_mode(is_open)
            .to_value()
            .map(|[t1, t2]| self.curve_in(t1, t2, res))
    }
}

impl<D: efd::EfdDim, M> Code<D> for M where
    M: Normalized<D>
        + CurveGen<D>
        + syn::SynBound
        + Statable
        + FromVectorized
        + IntoVectorized
        + 'static
{
}
