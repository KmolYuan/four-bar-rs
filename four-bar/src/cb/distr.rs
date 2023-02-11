use crate::*;

/// Uniform distribution of the [`NormFourBar`] type.
pub struct NormFbDistr;

impl mh::rand::Distribution<[NormFourBar; 2]> for NormFbDistr {
    fn sample<R: mh::rand::Rng + ?Sized>(&self, rng: &mut R) -> [NormFourBar; 2] {
        let v = crate::planar_syn::BOUND[..5]
            .iter()
            .map(|&[u, l]| rng.gen_range(u..l))
            .collect::<Vec<_>>();
        [false, true].map(|inv| NormFourBar::try_from(v.as_slice()).unwrap().with_inv(inv))
    }
}
