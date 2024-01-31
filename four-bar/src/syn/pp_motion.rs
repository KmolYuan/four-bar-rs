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
    pub curve: Vec<efd::Coord<D>>,
    /// Target pose
    pub pose: Vec<efd::Coord<D>>,
    /// Target position
    pub pos: Vec<f64>,
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
            .map(|(a, b)| std::array::from_fn(|i| b[i] - a[i]))
            .collect::<Vec<_>>();
        Self::from_uvec(curve, vectors, mode)
    }

    /// Create a new task from target (a curve and its unit vectors).
    pub fn from_uvec<C, V>(curve: C, vectors: V, mode: Mode) -> Self
    where
        C: efd::Curve<D>,
        V: efd::Curve<D>,
    {
        let mut curve = curve.to_curve();
        let mut pose = vectors.to_curve();
        let (pos, geo) = efd::get_target_pos(&curve, mode.is_target_open());
        let geo_inv = geo.inverse();
        geo_inv.transform_inplace(&mut curve);
        efd::GeoVar::from_rot(geo_inv.rot().clone()).transform_inplace(&mut pose);
        Self::new(Tar { curve, pose, pos, geo }, mode)
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
            let (efd, pose_efd) = efd::PosedEfd::from_uvec(c, v, is_open).into_inner();
            let geo = efd.as_geo().to(&self.tar.geo);
            let o_err = self.origin.map(|o| geo.trans().l2_norm(&o)).unwrap_or(0.);
            let s_err = self.scale.map(|s| (geo.scale() - s).abs()).unwrap_or(0.);
            let curve_err = zip(efd.generate_norm_by(&self.tar.pos), &self.tar.curve);
            let pose_err = zip(pose_efd.generate_norm_by(&self.tar.pos), &self.tar.pose);
            let err = curve_err
                .chain(pose_err)
                .map(|(a, b)| a.l2_norm(b))
                .fold(0., f64::max);
            let fb = fb.clone().trans_denorm(&geo);
            mh::Product::new(err.max(o_err).max(s_err), fb)
        })
    }
}
