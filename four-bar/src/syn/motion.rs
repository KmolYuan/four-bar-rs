use super::*;

/// Path generation task of planar four-bar linkage.
pub type MFbSyn = MotionSyn<MNormFourBar, 6, 2>;

/// Motion generation of a mechanism `M`.
pub struct MotionSyn<M, const N: usize, const D: usize>
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Target coefficients
    pub efd: efd::PosedEfd<D>,
    // Mode
    pub(crate) mode: Mode,
    // How many points need to be generated or compared
    res: usize,
    // Marker of the mechanism
    _marker: PhantomData<M>,
}

impl<M, const N: usize, const D: usize> MotionSyn<M, N, D>
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
        let efd = efd::PosedEfd::from_series(curve1, curve2, mode.is_target_open());
        Self::from_efd(efd, mode)
    }

    /// Create a new task from target (a curve and its unit vectors). The
    /// harmonic number is selected automatically.
    pub fn from_uvec<C, V>(curve: C, vectors: V, mode: Mode) -> Self
    where
        C: efd::Curve<D>,
        V: efd::Curve<D>,
    {
        let efd = efd::PosedEfd::from_uvec(curve, vectors, mode.is_target_open());
        Self::from_efd(efd, mode)
    }

    /// Create a new task from target EFD coefficients.
    pub fn from_efd(efd: efd::PosedEfd<D>, mode: Mode) -> Self {
        Self { efd, mode, res: 180, _marker: PhantomData }
    }

    /// Set the resolution during synthesis.
    pub fn res(self, res: usize) -> Self {
        assert!(res > 0);
        Self { res, ..self }
    }

    /// The harmonic used of target EFD.
    pub fn harmonic(&self) -> usize {
        self.efd.harmonic()
    }
}

impl<M, const N: usize, const D: usize> mh::ObjFunc for MotionSyn<M, N, D>
where
    M: SynBound<N, D> + mech::Normalized<D> + mech::PoseGen<D>,
    M::De: Default + Clone + Sync + Send + 'static,
    efd::Efd<D>: Sync + Send,
    efd::U<D>: efd::EfdDim<D>,
{
    type Fitness = mh::Product<M::De, f64>;

    fn fitness(&self, xs: &[f64]) -> Self::Fitness {
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
            iter.map(move |fb| (fb.pose_in(t1, t2, self.res), fb))
                .filter(|((c, _), _)| c.len() > 2)
                .map(|((c, v), fb)| {
                    let efd = efd::PosedEfd::from_uvec_harmonic(c, v, is_open, self.efd.harmonic());
                    let geo = efd.curve_efd().as_geo().to(self.efd.curve_efd().as_geo());
                    let fb = fb.clone().trans_denorm(&geo);
                    let err = efd.distance(&self.efd);
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
