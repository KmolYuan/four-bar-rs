use crate::io;
use four_bar::{mh::SolverBox, *};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

macro_rules! impl_method {
    ($(fn $method:ident, $sym:ident, $name:literal, $full_name:literal, $link:literal)+) => {
        pub(crate) const LIST: &'static [(&'static str, &'static str, fn() -> Self)] =
            &[$(($full_name, $name, Self::$method),)+];

        $(pub(crate) const fn $method() -> Self { Self::$sym(mh::$sym::new()) })+

        pub(crate) const fn name(&self) -> &'static str {
            match self { $(Self::$sym(_) => $full_name,)+ }
        }
        pub(crate) const fn abbr(&self) -> &'static str {
            match self { $(Self::$sym(_) => $name,)+ }
        }
        pub(crate) const fn link(&self) -> &'static str {
            match self { $(Self::$sym(_) => $link,)+ }
        }
    };
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(clap::Subcommand))]
pub(crate) enum SynAlg {
    De(mh::De),
    Fa(mh::Fa),
    Pso(mh::Pso),
    Rga(mh::Rga),
    Tlbo(mh::Tlbo),
}

impl Default for SynAlg {
    fn default() -> Self {
        Self::De(mh::De::default())
    }
}

impl SynAlg {
    impl_method! {
        fn de, De, "DE", "Differential Evolution", "https://en.wikipedia.org/wiki/Differential_evolution"
        fn fa, Fa, "FA", "Firefly Algorithm", "https://en.wikipedia.org/wiki/Firefly_algorithm"
        fn pso, Pso, "PSO", "Particle Swarm Optimization", "https://en.wikipedia.org/wiki/Particle_swarm_optimization"
        fn rga, Rga, "RGA", "Real-coded Genetic Algorithm", "https://en.wikipedia.org/wiki/Genetic_algorithm"
        fn tlbo, Tlbo, "TLBO", "Teaching Learning Based Optimization", "https://doi.org/10.1016/j.cad.2010.12.015"
    }

    pub(crate) fn build_solver<F: mh::ObjFunc>(self, f: F) -> SolverBox<'static, F> {
        match self {
            Self::De(s) => mh::Solver::build_boxed(s, f),
            Self::Fa(s) => mh::Solver::build_boxed(s, f),
            Self::Pso(s) => mh::Solver::build_boxed(s, f),
            Self::Rga(s) => mh::Solver::build_boxed(s, f),
            Self::Tlbo(s) => mh::Solver::build_boxed(s, f),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(clap::Args))]
pub(crate) struct SynCfg {
    /// Fix the seed to get a determined result, default to random
    #[cfg_attr(not(target_arch = "wasm32"), clap(short, long))]
    pub(crate) seed: Option<u64>,
    /// Number of generation
    #[cfg_attr(
        not(target_arch = "wasm32"),
        clap(alias = "iter"),
        clap(short, long, default_value_t = CFG_DEF.gen)
    )]
    pub(crate) gen: u64,
    /// Number of population (the fetch number in atlas)
    #[cfg_attr(not(target_arch = "wasm32"), clap(short, long, default_value_t = CFG_DEF.pop))]
    pub(crate) pop: usize,
    /// Number of the points (resolution) in curve production
    #[cfg_attr(not(target_arch = "wasm32"), clap(long, default_value_t = CFG_DEF.res))]
    pub(crate) res: usize,
    /// Specify the mechanism on the origin and unit scale
    #[cfg_attr(not(target_arch = "wasm32"), clap(long))]
    pub(crate) on_unit: bool,
    #[cfg_attr(not(target_arch = "wasm32"), clap(skip = CFG_DEF.mode))]
    pub(crate) mode: syn::Mode,
}

const CFG_DEF: SynCfg = SynCfg {
    seed: None,
    gen: 50,
    pop: 200,
    res: 180,
    on_unit: false,
    mode: syn::Mode::Closed,
};

impl Default for SynCfg {
    fn default() -> Self {
        CFG_DEF
    }
}

#[derive(Clone)]
pub(crate) enum Target<'a> {
    P(Cow<'a, [[f64; 2]]>, Cow<'a, atlas::FbAtlas>),
    S(Cow<'a, [[f64; 3]]>, Cow<'a, atlas::SFbAtlas>),
}

pub(crate) struct PathSynData<'a, M, const N: usize, const D: usize>
where
    syn::PathSyn<M, N, D>: mh::ObjFunc,
    efd::U<D>: efd::EfdDim<D>,
{
    pub(crate) s: SolverBox<'a, syn::PathSyn<M, N, D>>,
    pub(crate) atlas_fb: Option<(f64, M)>,
}

impl<'a, M, const N: usize, const D: usize> PathSynData<'a, M, N, D>
where
    syn::PathSyn<M, N, D>: mh::ObjFunc<Ys = mh::WithProduct<f64, M::De>>,
    M: atlas::Code<N, D>,
    M::De: Default + Clone + Sync + Send + 'static,
    efd::U<D>: efd::EfdDim<D>,
    efd::Efd<D>: Sync,
{
    fn new<S, C>(
        alg: SynAlg,
        target: &[[f64; D]],
        atlas: &atlas::Atlas<M, N, D>,
        cfg: SynCfg,
        stop: S,
        mut callback: C,
    ) -> Self
    where
        S: Fn() -> bool + Send + 'a,
        C: FnMut(f64, u64) + Send + 'a,
    {
        let SynCfg { seed, gen, pop, mode, res, on_unit } = cfg;
        let mut syn = syn::PathSyn::from_curve(target, mode).res(res);
        if on_unit {
            syn = syn.on_unit();
        }
        let s = alg
            .build_solver(syn)
            .seed(seed)
            .task(move |ctx| !stop() && ctx.gen >= gen)
            .callback(move |ctx| callback(ctx.best.get_eval(), ctx.gen));
        let mut data = Self { s, atlas_fb: None };
        if let Some(candi) = (!mode.is_partial())
            .then(|| atlas.fetch_raw(target, mode.is_target_open(), pop))
            .filter(|candi| !candi.is_empty())
        {
            data.atlas_fb = Some(candi[0].clone());
            let pool_y = candi
                .iter()
                .map(|(f, fb)| mh::WithProduct::new(*f, fb.clone().denormalize()))
                .collect();
            let pool = candi
                .into_iter()
                .map(|(_, fb)| fb.into_vectorized().0)
                .collect();
            data.s = data.s.init_pool(mh::Pool::Ready { pool, pool_y });
        } else {
            data.s = data.s.pop_num(pop);
        }
        data
    }

    fn solve(self) -> (M::De, Option<(f64, M)>) {
        (self.s.solve().into_result(), self.atlas_fb)
    }
}

pub(crate) enum Solver<'a> {
    FbSyn(PathSynData<'a, NormFourBar, 5, 2>),
    SFbSyn(PathSynData<'a, SNormFourBar, 6, 3>),
}

impl<'a> Solver<'a> {
    pub(crate) fn new<S, C>(alg: SynAlg, target: Target, cfg: SynCfg, stop: S, callback: C) -> Self
    where
        S: Fn() -> bool + Send + 'a,
        C: FnMut(f64, u64) + Send + 'a,
    {
        match target {
            Target::P(target, atlas) => {
                Self::FbSyn(PathSynData::new(alg, &target, &atlas, cfg, stop, callback))
            }
            Target::S(target, atlas) => {
                Self::SFbSyn(PathSynData::new(alg, &target, &atlas, cfg, stop, callback))
            }
        }
    }

    pub(crate) fn solve(self) -> io::Fb {
        match self {
            Self::FbSyn(s) => io::Fb::P(s.solve().0),
            Self::SFbSyn(s) => io::Fb::S(s.solve().0),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn solve_verbose(self) -> Result<(f64, usize, SolvedFb), ndarray::ShapeError> {
        macro_rules! impl_solve {
            ($syn:ident, $s:ident, $atlas_fb:ident) => {{
                let s = $s.solve();
                let func = s.func();
                let h = func.harmonic();
                let tar = func.tar.clone();
                let (err, fb) = s.into_err_result();
                Ok((err, h, SolvedFb::$syn(fb, tar, $atlas_fb)))
            }};
        }
        match self {
            Self::FbSyn(PathSynData { s, atlas_fb }) => impl_solve!(P, s, atlas_fb),
            Self::SFbSyn(PathSynData { s, atlas_fb }) => impl_solve!(S, s, atlas_fb),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) enum SolvedFb {
    P(FourBar, efd::Efd2, Option<(f64, NormFourBar)>),
    S(SFourBar, efd::Efd3, Option<(f64, SNormFourBar)>),
}
