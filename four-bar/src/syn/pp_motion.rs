use super::*;

/// Precise-point motion generation task of planar four-bar linkage.
pub type MFbPPSyn = PrecisePointMotionSyn<MFourBar, 6, 2>;

/// Precise-point motion generation of a mechanism `M`.
pub struct PrecisePointMotionSyn<M, const N: usize, const D: usize>
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Target data
    pub tar: PrecisePointTarget<D>,
    /// Target pose
    pub tar_pose: Vec<efd::Coord<D>>,
    // Mode
    mode: Mode,
    // How many points need to be generated and compared
    res: usize,
    // Marker of the mechanism
    _marker: PhantomData<M>,
}

impl<M, const N: usize, const D: usize> PrecisePointMotionSyn<M, N, D>
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
        let vectors = curve
            .iter()
            .zip(curve2.as_curve())
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
        let curve = curve.to_curve();
        let (pos, geo) = efd::get_target_pos(&curve, mode.is_target_open());
        Self {
            tar: PrecisePointTarget { curve, pos, geo },
            tar_pose: vectors.to_curve(),
            mode,
            res: 180,
            _marker: PhantomData,
        }
    }

    /// Set the resolution during synthesis.
    pub fn res(self, res: usize) -> Self {
        assert!(res > 0);
        Self { res, ..self }
    }
}

impl<M, const N: usize, const D: usize> mh::Bounded for PrecisePointMotionSyn<M, N, D>
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

impl<M, const N: usize, const D: usize> mh::ObjFunc for PrecisePointMotionSyn<M, N, D>
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
            let (mut efd, pose_efd) = efd::PosedEfd::from_uvec(c, v, is_open).into_inner();
            let geo = efd.as_geo().to(&self.tar.geo);
            *efd.as_geo_mut() = geo.clone();
            let err1 = efd
                .generate_by(&self.tar.pos)
                .iter()
                .zip(&self.tar.curve)
                .map(|(a, b)| a.l2_norm(b))
                .sum::<f64>();
            // FIXME: Geometry maybe not right
            let err2 = pose_efd
                .generate_by(&self.tar.pos)
                .iter()
                .zip(&self.tar_pose)
                .map(|(a, b)| a.l2_norm(b))
                .sum::<f64>();
            let fb = fb.clone().trans_denorm(&geo);
            mh::Product::new(err1 + err2, fb)
        })
    }
}
