use super::*;

/// Precise-point path generation task of planar four-bar linkage.
pub type FbPPSyn = PrecisePointPathSyn<NormFourBar, 5, 2>;
/// Precise-point path generation task of spherical four-bar linkage.
pub type SFbPPSyn = PrecisePointPathSyn<SNormFourBar, 6, 3>;

/// Target data of precise-point path generation.
pub struct PrecisePointTarget<const D: usize>
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

/// Precise-point path generation of a mechanism `M`.
pub struct PrecisePointPathSyn<M, const N: usize, const D: usize>
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Target data
    pub tar: PrecisePointTarget<D>,
    // Mode
    mode: Mode,
    // How many points need to be generated and compared
    res: usize,
    // Constrain the origin of the mechanism
    origin: Option<efd::Coord<D>>,
    // Constrain the scale of the mechanism
    scale: Option<f64>,
    // Marker of the mechanism
    _marker: PhantomData<M>,
}

impl<M, const N: usize, const D: usize> PrecisePointPathSyn<M, N, D>
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
        Self {
            tar: PrecisePointTarget { curve, pos, geo },
            mode,
            res: 180,
            origin: None,
            scale: None,
            _marker: PhantomData,
        }
    }

    /// Set the resolution during synthesis.
    pub fn res(self, res: usize) -> Self {
        assert!(res > 0);
        Self { res, ..self }
    }

    /// Specify the mechanism is on origin and unit scale.
    pub fn on_unit(self) -> Self {
        self.origin([0.; D]).scale(1.)
    }

    /// Specify the origin of the mechanism.
    pub fn origin(self, origin: efd::Coord<D>) -> Self {
        Self { origin: Some(origin), ..self }
    }

    /// Specify the scale of the mechanism.
    pub fn scale(self, scale: f64) -> Self {
        assert!(scale > 0.);
        Self { scale: Some(scale), ..self }
    }
}

impl<M, const N: usize, const D: usize> mh::Bounded for PrecisePointPathSyn<M, N, D>
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

impl<M, const N: usize, const D: usize> mh::ObjFunc for PrecisePointPathSyn<M, N, D>
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
            let o_err = self.origin.map(|o| geo.trans().l2_norm(&o)).unwrap_or(0.);
            let s_err = self.scale.map(|s| (geo.scale() - s).abs()).unwrap_or(0.);
            let err = std::iter::zip(efd.generate_norm_by(&self.tar.pos), &self.tar.curve)
                .map(|(a, b)| a.l2_norm(b))
                .fold(0., f64::max);
            let fb = fb.clone().trans_denorm(&geo);
            mh::Product::new(o_err.max(s_err).max(err), fb)
        })
    }
}
