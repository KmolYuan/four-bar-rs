use crate::*;
use mh::rand::{Distribution, Rng};

/// Uniform distribution of the [`NormFourBar`] type.
pub struct NormFbDistr;

impl Distribution<[NormFourBar; 2]> for NormFbDistr {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [NormFourBar; 2] {
        let v = crate::planar_syn::BOUND[..5]
            .iter()
            .map(|&[u, l]| rng.gen_range(u..l))
            .collect::<Vec<_>>();
        [false, true].map(|inv| NormFourBar::try_from(v.as_slice()).unwrap().with_inv(inv))
    }
}

/// Uniform distribution of the [`SNormFourBar`] type.
pub struct SNormFbDistr;

impl Distribution<[SNormFourBar; 2]> for SNormFbDistr {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [SNormFourBar; 2] {
        let v = crate::spherical_syn::BOUND[..6]
            .iter()
            .map(|&[u, l]| rng.gen_range(u..l))
            .collect::<Vec<_>>();
        [false, true].map(|inv| SNormFourBar::try_from(v.as_slice()).unwrap().with_inv(inv))
    }
}
