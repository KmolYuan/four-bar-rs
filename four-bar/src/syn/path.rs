use super::*;

/// Path generation task of planar four-bar linkage.
pub type FbSyn = PathSyn<NormFourBar, 5, 2>;
/// Path generation task of spherical four-bar linkage.
pub type SFbSyn = PathSyn<SNormFourBar, 6, 3>;
/// Path generation of a mechanism `M`.
pub type PathSyn<M, const N: usize, const D: usize> = Syn<efd::Efd<D>, M, N, D>;

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
        Self::new(efd, mode)
    }

    /// The harmonic used of target EFD.
    pub fn harmonic(&self) -> usize {
        self.tar.harmonic()
    }
}

impl<M, const N: usize, const D: usize> mh::Bounded for PathSyn<M, N, D>
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

impl<M, const N: usize, const D: usize> mh::ObjFunc for PathSyn<M, N, D>
where
    M: SynBound<N> + mech::Normalized<D> + mech::CurveGen<D>,
    M::De: Default + Clone + Sync + Send + 'static,
    efd::Efd<D>: Sync + Send,
    efd::U<D>: efd::EfdDim<D>,
{
    type Ys = mh::Product<f64, M::De>;

    fn fitness(&self, xs: &[f64]) -> Self::Ys {
        let is_open = self.mode.is_target_open();
        let get_series = |fb: &M, start, end| {
            let curve = fb.curve_in(start, end, self.res);
            (curve.len() > 2).then_some(curve)
        };
        impl_fitness(self.mode, xs, get_series, |(c, fb)| {
            let efd = efd::Efd::from_curve_harmonic(c, is_open, self.harmonic());
            let geo = efd.as_geo().to(self.tar.as_geo());
            let fb = fb.clone().trans_denorm(&geo);
            mh::Product::new(efd.err(&self.tar).max(self.unit_err(&geo)), fb)
        })
    }
}
