use super::*;

/// Path generation task of planar four-bar linkage.
pub type FbSyn = PathSyn<NormFourBar, 5, 2>;
/// Path generation task of spherical four-bar linkage.
pub type SFbSyn = PathSyn<SNormFourBar, 6, 3>;

/// Path generation of a mechanism `M`.
pub struct PathSyn<M, const N: usize, const D: usize>
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Target coefficients
    pub efd: efd::Efd<D>,
    // Mode
    pub(crate) mode: Mode,
    // How many points need to be generated or compared
    res: usize,
    // Constrain the origin of the mechanism
    origin: Option<efd::Coord<D>>,
    // Constrain the scale of the mechanism
    scale: Option<f64>,
    // Marker of the mechanism
    _marker: PhantomData<M>,
}

impl<M, const N: usize, const D: usize> PathSyn<M, N, D>
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Create a new task from target curve. The harmonic number is selected
    /// automatically.
    pub fn from_curve<C>(curve: C, mode: Mode) -> Self
    where
        C: efd::Curve<D>,
    {
        Self::from_efd(efd::Efd::from_curve(curve, mode.is_target_open()), mode)
    }

    /// Create a new task from target EFD coefficients.
    pub fn from_efd(efd: efd::Efd<D>, mode: Mode) -> Self {
        Self {
            efd,
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

    /// The harmonic used of target EFD.
    pub fn harmonic(&self) -> usize {
        self.efd.harmonic()
    }
}

impl<M, const N: usize, const D: usize> mh::ObjFunc for PathSyn<M, N, D>
where
    efd::Rot<D>: Sync + Send,
    efd::Coord<D>: efd::Distance + Sync + Send,
    M: SynBound<N>
        + mech::Statable
        + mech::FromVectorized<N>
        + mech::Normalized<D>
        + mech::CurveGen<D>,
    M::De: Default + Clone + Sync + Send + 'static,
    efd::U<D>: efd::EfdDim<D>,
{
    type Fitness = mh::Product<M::De, f64>;

    fn fitness(&self, xs: &[f64]) -> Self::Fitness {
        use efd::Distance as _;
        #[cfg(feature = "rayon")]
        use mh::rayon::prelude::*;
        let mut fb = M::from_vectorized_s1(slice_to_array(xs));
        fb.set_to_planar_loop();
        let (bound, states) =
            fb.to_bound_states_filter(|a| a.check_mode(self.mode.is_result_open()));
        let is_open = self.mode.is_target_open();
        let f = |[t1, t2]: [f64; 2]| {
            #[cfg(feature = "rayon")]
            let iter = states.par_iter();
            #[cfg(not(feature = "rayon"))]
            let iter = states.iter();
            iter.map(move |fb| (fb.curve_in(t1, t2, self.res), fb))
                .filter(|(c, _)| c.len() > 2)
                .map(|(c, fb)| {
                    let efd = efd::Efd::from_curve_harmonic(c, is_open, self.efd.harmonic());
                    let geo = efd.as_geo().to(self.efd.as_geo());
                    let fb = fb.clone().trans_denorm(&geo);
                    let o_err = match &self.origin {
                        Some(o) => geo.trans().l2_norm(o),
                        None => 0.,
                    };
                    let s_err = match self.scale {
                        Some(s) => (geo.scale() - s).abs(),
                        None => 0.,
                    };
                    let err = efd.distance(&self.efd).max(o_err).max(s_err);
                    mh::Product::new(err, fb)
                })
        };
        match self.mode {
            Mode::Closed | Mode::Open => bound
                .check_min()
                .to_value()
                .and_then(|t| f(t).min_by(|a, b| a.partial_cmp(b).unwrap()))
                .unwrap_or_else(infeasible),
            Mode::Partial if !bound.is_valid() => infeasible(),
            Mode::Partial => {
                let bound = mech::AngleBound::open_and_rev_at(xs[N], xs[N + 1]);
                #[cfg(feature = "rayon")]
                let iter = bound.into_par_iter();
                #[cfg(not(feature = "rayon"))]
                let iter = bound.into_iter();
                iter.filter_map(|b| b.check_min().to_value())
                    .flat_map(f)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or_else(infeasible)
            }
        }
    }
}
