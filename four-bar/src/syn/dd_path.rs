use super::*;

/// Distance-discrepancy path generation task of planar four-bar linkage.
pub type FbDDSyn = DDPathSyn<NormFourBar, 5, 2>;
/// Distance-discrepancy path generation task of spherical four-bar linkage.
pub type SFbDDSyn = DDPathSyn<SNormFourBar, 6, 3>;
/// Distance-discrepancy path generation of a mechanism `M`.
pub type DDPathSyn<M, const N: usize, const D: usize> = Syn<efd::PathSig<D>, M, N, D>;

impl<M, const N: usize, const D: usize> DDPathSyn<M, N, D>
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Create a new task from target curve.
    pub fn from_curve<C>(curve: C, mode: Mode) -> Self
    where
        C: efd::Curve<D>,
    {
        let sig = efd::PathSig::new(curve, mode.is_target_open());
        Self::new(sig, mode)
    }
}

impl<M, const N: usize, const D: usize> mh::Bounded for DDPathSyn<M, N, D>
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

impl<M, const N: usize, const D: usize> mh::ObjFunc for DDPathSyn<M, N, D>
where
    M: SynBound<N> + mech::Normalized<D> + mech::CurveGen<D>,
    M::De: Default + Clone + Sync + Send + 'static,
    efd::Rot<D>: Sync + Send,
    efd::U<D>: efd::EfdDim<D>,
{
    type Ys = mh::WithProduct<f64, M::De>;

    fn fitness(&self, xs: &[f64]) -> Self::Ys {
        let is_open = self.mode.is_target_open();
        let get_series = |fb: &M, start, end| {
            let curve = fb.curve_in(start, end, self.res);
            (curve.len() > 2).then_some(curve)
        };
        impl_fitness(self.mode, xs, get_series, |(c, fb)| {
            let efd = efd::Efd::from_curve(c, is_open);
            let geo = efd.as_geo().to(self.tar.as_geo());
            let fb = fb.clone().trans_denorm(&geo);
            mh::WithProduct::new(efd.err_sig(&self.tar).max(self.unit_err(&geo)), fb)
        })
    }
}
