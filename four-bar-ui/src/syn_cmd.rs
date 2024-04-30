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
pub(crate) enum Target<'a, 'b> {
    Fb {
        target: Cow<'a, [[f64; 2]]>,
        target_fb: Option<FourBar>,
        atlas: Option<&'b atlas::FbAtlas>,
    },
    MFb {
        target: Cow<'a, [([f64; 2], [f64; 2])]>,
        target_fb: Option<MFourBar>,
    },
    SFb {
        target: Cow<'a, [[f64; 3]]>,
        target_fb: Option<SFourBar>,
        atlas: Option<&'b atlas::SFbAtlas>,
    },
}

impl<'a, 'b> Target<'a, 'b> {
    pub(crate) fn fb(
        target: Cow<'a, [[f64; 2]]>,
        target_fb: Option<FourBar>,
        atlas: Option<&'b atlas::FbAtlas>,
    ) -> Self {
        Self::Fb { target, target_fb, atlas }
    }

    pub(crate) fn mfb(
        target: Cow<'a, [([f64; 2], [f64; 2])]>,
        target_fb: Option<MFourBar>,
    ) -> Self {
        Self::MFb { target, target_fb }
    }

    pub(crate) fn sfb(
        target: Cow<'a, [[f64; 3]]>,
        target_fb: Option<SFourBar>,
        atlas: Option<&'b atlas::SFbAtlas>,
    ) -> Self {
        Self::SFb { target, target_fb, atlas }
    }
}

pub(crate) struct PathSynData<'a, M, const N: usize, const D: usize>
where
    M: mech::Normalized<D>,
    syn::PathSyn<M, N, D>: mh::ObjFunc,
    efd::U<D>: efd::EfdDim<D>,
{
    pub(crate) s: SolverBox<'a, syn::PathSyn<M, N, D>>,
    pub(crate) target: Cow<'a, [[f64; D]]>,
    pub(crate) target_fb: Option<M::De>,
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
        target: Cow<'a, [[f64; D]]>,
        target_fb: Option<M::De>,
        atlas: Option<&atlas::Atlas<M, N, D>>,
        cfg: SynCfg,
        stop: S,
        mut callback: C,
    ) -> Self
    where
        S: Fn() -> bool + Send + 'a,
        C: FnMut(f64, u64) + Send + 'a,
    {
        let SynCfg { seed, gen, pop, mode, res, on_unit } = cfg;
        let mut syn = syn::PathSyn::from_curve(&target, mode).res(res);
        if on_unit {
            syn = syn.on_unit();
        }
        let mut s = alg
            .build_solver(syn)
            .seed(seed)
            .task(move |ctx| !stop() && ctx.gen >= gen)
            .callback(move |ctx| callback(ctx.best.get_eval(), ctx.gen));
        let Some(atlas) = atlas else {
            return Self { s, target, target_fb, atlas_fb: None };
        };
        let mut atlas_fb = None;
        if let Some(candi) = (!mode.is_partial())
            .then(|| atlas.fetch_raw(&target, mode.is_target_open(), pop))
            .filter(|candi| !candi.is_empty())
        {
            atlas_fb = Some(candi[0].clone());
            let pool_y = candi
                .iter()
                .map(|(f, fb)| mh::WithProduct::new(*f, fb.clone().denormalize()))
                .collect();
            let pool = candi
                .into_iter()
                .map(|(_, fb)| fb.into_vectorized().0)
                .collect();
            s = s.init_pool(mh::Pool::Ready { pool, pool_y });
        } else {
            s = s.pop_num(pop);
        }
        Self { s, target, target_fb, atlas_fb }
    }

    fn solve(self) -> (M::De, Option<(f64, M)>) {
        (self.s.solve().into_result(), self.atlas_fb)
    }
}

pub(crate) enum Solver<'a> {
    Fb(PathSynData<'a, NormFourBar, 5, 2>),
    SFb(PathSynData<'a, SNormFourBar, 6, 3>),
}

impl<'a> Solver<'a> {
    pub(crate) fn new<S, C>(
        alg: SynAlg,
        target: Target<'a, '_>,
        cfg: SynCfg,
        stop: S,
        callback: C,
    ) -> Self
    where
        S: Fn() -> bool + Send + 'a,
        C: FnMut(f64, u64) + Send + 'a,
    {
        match target {
            Target::Fb { target, target_fb, atlas } => Self::Fb(PathSynData::new(
                alg, target, target_fb, atlas, cfg, stop, callback,
            )),
            Target::MFb { target, target_fb } => {
                // TODO: Implement this!
                unimplemented!("synthesis with {target:?} {target_fb:?}")
            }
            Target::SFb { target, target_fb, atlas } => Self::SFb(PathSynData::new(
                alg, target, target_fb, atlas, cfg, stop, callback,
            )),
        }
    }

    pub(crate) fn solve(self) -> io::Fb {
        match self {
            Self::Fb(s) => io::Fb::P(s.solve().0),
            Self::SFb(s) => io::Fb::S(s.solve().0),
        }
    }
}
