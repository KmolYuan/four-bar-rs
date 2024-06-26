use super::*;
use std::iter::zip;

/// Distance-discrepancy motion generation task of planar four-bar linkage.
pub type MFbDDSyn = DDMotionSyn<MNormFourBar, 6, 2>;
/// Distance-discrepancy motion generation of a mechanism `M`.
pub type DDMotionSyn<M, const N: usize, const D: usize> = Syn<efd::posed::MotionSig<D>, M, N, D>;

impl<M, const N: usize, const D: usize> DDMotionSyn<M, N, D>
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
        assert_ne!(mode, Mode::Closed, "Closed mode is not supported");
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
        let sig = efd::MotionSig::new(curve, vectors, mode.is_target_open());
        Self::new(sig, mode)
    }
}

impl<M, const N: usize, const D: usize> mh::Bounded for DDMotionSyn<M, N, D>
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

impl<M, const N: usize, const D: usize> mh::ObjFunc for DDMotionSyn<M, N, D>
where
    M: SynBound<N> + mech::Normalized<D> + mech::PoseGen<D>,
    M::De: Default + Clone + Sync + Send + 'static,
    efd::Rot<D>: Sync + Send,
    efd::U<D>: efd::EfdDim<D>,
{
    type Ys = mh::WithProduct<f64, M::De>;

    fn fitness(&self, xs: &[f64]) -> Self::Ys {
        let is_open = self.mode.is_target_open();
        let get_series = |fb: &M, start, end| {
            let (curve, pose) = fb.pose_in(start, end, self.res);
            (curve.len() > 2).then_some((curve, pose))
        };
        impl_fitness(self.mode, xs, get_series, |((c, v), fb)| {
            let efd = efd::PosedEfd::from_uvec(c, v, is_open);
            let geo = efd.as_curve().as_geo().to(self.tar.as_geo());
            let fb = fb.clone().trans_denorm(&geo);
            use efd::Distance as _;
            let curve = zip(
                efd.as_curve().recon_norm_by(self.tar.as_t()),
                &self.tar.curve,
            )
            .map(|(a, b)| a.l2_err(b))
            .fold(0., f64::max);
            let pose = zip(efd.as_pose().recon_by(self.tar.as_t()), &self.tar.vectors)
                .map(|(a, b)| a.l2_err(b))
                .fold(0., f64::max);
            let err = curve.max(pose).max(self.unit_err(&geo));
            mh::WithProduct::new(err, fb)
        })
    }
}

#[inline]
fn renorm<const D: usize>(v: [f64; D]) -> [f64; D] {
    use efd::Distance as _;
    let norm = v.l2_err(&[0.; D]);
    v.map(|x| x / norm)
}
