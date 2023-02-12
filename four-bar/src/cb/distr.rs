use crate::*;
use efd::*;
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

pub trait Code<const N: usize, const DIM: usize>: Sized {
    type Distr: Distribution<[Self; 2]> + Sync;
    type Trans: Trans;
    type UnNorm;

    fn distr() -> Self::Distr;
    fn from_code(code: [f64; N], inv: bool) -> Self;
    fn to_code(self) -> ([f64; N], bool);
    fn is_open(&self) -> bool;
    fn curve(&self, res: usize) -> Option<Vec<<Self::Trans as Trans>::Coord>>;
    fn unnorm(self, trans: Transform<Self::Trans>) -> Self::UnNorm;
}

impl Code<5, 2> for NormFourBar {
    type Distr = NormFbDistr;
    type UnNorm = FourBar;
    type Trans = T2;

    fn distr() -> Self::Distr {
        NormFbDistr
    }

    fn from_code(code: [f64; 5], inv: bool) -> Self {
        Self::new(code, inv)
    }

    fn to_code(self) -> ([f64; 5], bool) {
        (self.as_array(), self.inv)
    }

    fn is_open(&self) -> bool {
        self.ty().is_open_curve()
    }

    fn curve(&self, res: usize) -> Option<Vec<[f64; 2]>> {
        self.angle_bound()
            .filter(|[t1, t2]| t2 - t1 > crate::planar_syn::MIN_ANGLE)
            .map(|[t1, t2]| self.curve_in(t1, t2, res))
    }

    fn unnorm(self, trans: Transform<Self::Trans>) -> Self::UnNorm {
        FourBar::from_norm_trans(self, &trans)
    }
}
