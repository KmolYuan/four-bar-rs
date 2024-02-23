use super::*;
use std::iter::zip;

/// Precise-point motion generation task of planar four-bar linkage.
pub type MFbPPSyn = PPMotionSyn<MNormFourBar, 6, 2>;
/// Precise-point motion generation of a mechanism `M`.
pub type PPMotionSyn<M, const N: usize, const D: usize> = Syn<Tar<D>, M, N, D>;

/// Target data of precise-point path generation.
pub struct Tar<const D: usize>
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Target curve coordinates
    pub sig: Vec<efd::Coord<D>>,
    /// Target position
    pub t: Vec<f64>,
    /// Target geometry
    pub geo: efd::GeoVar<efd::Rot<D>, D>,
}

impl<M, const N: usize, const D: usize> PPMotionSyn<M, N, D>
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Create a new task from target (two series of points). The harmonic
    /// number is selected automatically.
    pub fn from_series<C1, C2>(curve1: C1, curve2: C2, mode: Mode) -> Self
    where
        C1: efd::Curve<D>,
        C2: efd::Curve<D>,
    {
        let curve = curve1.as_curve();
        let vectors = zip(curve, curve2.as_curve())
            .map(|(a, b)| renorm(std::array::from_fn(|i| b[i] - a[i])))
            .collect::<Vec<_>>();
        Self::from_uvec(curve, vectors, mode)
    }

    /// Create a new task from target (a curve and its unit vectors).
    pub fn from_uvec<C, V>(curve: C, vectors: V, mode: Mode) -> Self
    where
        C: efd::Curve<D>,
        V: efd::Curve<D>,
    {
        let (sig, t, geo) = efd::posed::path_signature(curve, vectors, mode.is_target_open());
        Self::new(Tar { sig, t, geo }, mode)
    }
}

impl<M, const N: usize, const D: usize> mh::Bounded for PPMotionSyn<M, N, D>
where
    Self: mh::ObjFunc,
    M: mech::FromVectorized<N>,
    efd::U<D>: efd::EfdDim<D>,
{
    #[inline]
    fn bound(&self) -> &[[f64; 2]] {
        if self.mode == Mode::Partial {
            M::BOUND_PARTIAL
        } else {
            &M::BOUND
        }
    }
}

impl<M, const N: usize, const D: usize> mh::ObjFunc for PPMotionSyn<M, N, D>
where
    M: SynBound<N> + mech::Normalized<D> + mech::PoseGen<D>,
    M::De: Default + Clone + Sync + Send + 'static,
    efd::Rot<D>: Sync + Send,
    efd::U<D>: efd::EfdDim<D>,
{
    type Fitness = mh::Product<M::De, f64>;

    fn fitness(&self, xs: &[f64]) -> Self::Fitness {
        let is_open = self.mode.is_target_open();
        let get_series = |fb: &M, start, end| {
            let (curve, pose) = fb.pose_in(start, end, self.res);
            (curve.len() > 2).then_some((curve, pose))
        };
        impl_fitness(self.mode, xs, get_series, |((c, v), fb)| {
            use efd::Distance as _;
            let efd = efd::PosedEfd::from_uvec(c, v, is_open);
            let geo = efd.as_geo().to(&self.tar.geo);
            let err = zip(efd.generate_norm_by(&self.tar.t), &self.tar.sig)
                .map(|(a, b)| a.l2_err(b))
                .fold(0., f64::max);
            let fb = fb.clone().trans_denorm(&geo);
            mh::Product::new(err.max(self.unit_err(&geo)), fb)
        })
    }
}

#[inline]
fn renorm<const D: usize>(v: [f64; D]) -> [f64; D] {
    use efd::Distance as _;
    let norm = v.l2_err(&[0.; D]);
    v.map(|x| x / norm)
}
