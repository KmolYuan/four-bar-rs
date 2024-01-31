use super::*;

/// Precise-point path generation task of planar four-bar linkage.
pub type FbPPSyn = PPPathSyn<NormFourBar, 5, 2>;
/// Precise-point path generation task of spherical four-bar linkage.
pub type SFbPPSyn = PPPathSyn<SNormFourBar, 6, 3>;
/// Precise-point path generation of a mechanism `M`.
pub type PPPathSyn<M, const N: usize, const D: usize> = Syn<Tar<D>, M, N, D>;

/// Target data of precise-point path generation.
pub struct Tar<const D: usize>
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Target curve coordinates
    pub curve: Vec<efd::Coord<D>>,
    /// Target position
    pub pos: Vec<f64>,
    /// Target geometry
    pub geo: efd::GeoVar<efd::Rot<D>, D>,
}

impl<M, const N: usize, const D: usize> PPPathSyn<M, N, D>
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Create a new task from target curve.
    pub fn from_curve<C>(curve: C, mode: Mode) -> Self
    where
        C: efd::Curve<D>,
    {
        let mut curve = curve.to_curve();
        let (pos, geo) = efd::get_target_pos(&curve, mode.is_target_open());
        geo.inverse().transform_inplace(&mut curve);
        Self::new(Tar { curve, pos, geo }, mode)
    }
}

impl<M, const N: usize, const D: usize> mh::Bounded for PPPathSyn<M, N, D>
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

impl<M, const N: usize, const D: usize> mh::ObjFunc for PPPathSyn<M, N, D>
where
    M: SynBound<N> + mech::Normalized<D> + mech::CurveGen<D>,
    M::De: Default + Clone + Sync + Send + 'static,
    efd::Rot<D>: Sync + Send,
    efd::U<D>: efd::EfdDim<D>,
{
    type Fitness = mh::Product<M::De, f64>;

    fn fitness(&self, xs: &[f64]) -> Self::Fitness {
        let is_open = self.mode.is_target_open();
        let get_series = |fb: &M, start, end| {
            let curve = fb.curve_in(start, end, self.res);
            (curve.len() > 2).then_some(curve)
        };
        impl_fitness(self.mode, xs, get_series, |(c, fb)| {
            use efd::Distance as _;
            let efd = efd::Efd::from_curve(c, is_open);
            let geo = efd.as_geo().to(&self.tar.geo);
            let err = std::iter::zip(efd.generate_norm_by(&self.tar.pos), &self.tar.curve)
                .map(|(a, b)| a.l2_norm(b))
                .fold(0., f64::max);
            let fb = fb.clone().trans_denorm(&geo);
            mh::Product::new(err.max(self.unit_err(&geo)), fb)
        })
    }
}
