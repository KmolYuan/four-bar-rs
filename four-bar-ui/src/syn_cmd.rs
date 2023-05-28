use crate::io;
use four_bar::*;
use serde::{Deserialize, Serialize};

macro_rules! impl_method {
    ($(fn $method:ident, $sym:ident, $name:literal, $full_name:literal, $link:literal)+) => {
        pub(crate) const LIST: &[(&'static str, &'static str, fn() -> Self)] =
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
pub(crate) enum SynMethod {
    De(mh::De),
    Fa(mh::Fa),
    Pso(mh::Pso),
    Rga(mh::Rga),
    Tlbo(mh::Tlbo),
}

impl Default for SynMethod {
    fn default() -> Self {
        Self::De(mh::De::default())
    }
}

impl SynMethod {
    impl_method! {
        fn de, De, "DE", "Differential Evolution", "https://en.wikipedia.org/wiki/Differential_evolution"
        fn fa, Fa, "FA", "Firefly Algorithm", "https://en.wikipedia.org/wiki/Firefly_algorithm"
        fn pso, Pso, "PSO", "Particle Swarm Optimization", "https://en.wikipedia.org/wiki/Particle_swarm_optimization"
        fn rga, Rga, "RGA", "Real-coded Genetic Algorithm", "https://en.wikipedia.org/wiki/Genetic_algorithm"
        fn tlbo, Tlbo, "TLBO", "Teaching Learning Based Optimization", "https://doi.org/10.1016/j.cad.2010.12.015"
    }

    pub(crate) fn build_solver<F>(self, f: F) -> mh::utility::SolverBuilder<'static, F>
    where
        F: mh::ObjFunc,
    {
        match self {
            Self::De(s) => mh::Solver::build(s, f),
            Self::Fa(s) => mh::Solver::build(s, f),
            Self::Pso(s) => mh::Solver::build(s, f),
            Self::Rga(s) => mh::Solver::build(s, f),
            Self::Tlbo(s) => mh::Solver::build(s, f),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub(crate) struct SynConfig {
    pub(crate) seed: Option<u64>,
    pub(crate) gen: u64,
    pub(crate) pop: usize,
    pub(crate) mode: syn::Mode,
}

impl Default for SynConfig {
    fn default() -> Self {
        Self {
            seed: None,
            gen: 50,
            pop: 200,
            mode: syn::Mode::Closed,
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) enum Target {
    P(Vec<[f64; 2]>, #[serde(skip)] cb::FbCodebook),
    S(Vec<[f64; 3]>, #[serde(skip)] cb::SFbCodebook),
}

impl Default for Target {
    fn default() -> Self {
        Self::P(Vec::new(), Default::default())
    }
}

impl Target {
    pub(crate) fn set_curve(&mut self, curve: io::Curve) {
        match (curve, self) {
            (io::Curve::P(c), Self::P(t, _)) => *t = c,
            (io::Curve::S(c), Self::S(t, _)) => *t = c,
            (io::Curve::P(c), t @ Self::S(_, _)) => *t = Self::P(c, Default::default()),
            (io::Curve::S(c), t @ Self::P(_, _)) => *t = Self::S(c, Default::default()),
        }
    }

    pub(crate) fn set_cb(&mut self, cb: io::Cb) -> Result<(), mh::ndarray::ShapeError> {
        match (cb, self) {
            (io::Cb::P(c), Self::P(_, t)) => t.merge_inplace(&c)?,
            (io::Cb::S(c), Self::S(_, t)) => t.merge_inplace(&c)?,
            (io::Cb::P(c), t @ Self::S(_, _)) => *t = Self::P(Vec::new(), c),
            (io::Cb::S(c), t @ Self::P(_, _)) => *t = Self::S(Vec::new(), c),
        }
        Ok(())
    }

    pub(crate) fn has_target(&self) -> bool {
        match self {
            Self::P(t, _) => !t.is_empty(),
            Self::S(t, _) => !t.is_empty(),
        }
    }
}

type FbBuilder = mh::utility::SolverBuilder<'static, syn::FbSyn>;
type SFbBuilder = mh::utility::SolverBuilder<'static, syn::SFbSyn>;

pub(crate) enum Solver {
    FbSyn(FbBuilder, Option<(f64, NormFourBar)>),
    SFbSyn(SFbBuilder, Option<(f64, SNormFourBar)>),
}

impl Solver {
    pub(crate) fn new<C>(method: SynMethod, target: Target, cfg: SynConfig, f: C) -> Self
    where
        C: Fn(f64, u64) + Send + 'static,
    {
        let SynConfig { seed, gen, pop, mode } = cfg;
        macro_rules! impl_solve {
            ($target:ident, $cb:ident, $syn:ident) => {{
                let mut s = method
                    .build_solver(syn::$syn::from_curve(&$target, mode))
                    .seed(seed)
                    .task(move |ctx| ctx.gen >= gen)
                    .callback(move |ctx| f(ctx.best_f.fitness(), ctx.gen));
                let mut cb_fb = None;
                if let Some(candi) = matches!(mode, syn::Mode::Closed | syn::Mode::Open)
                    .then(|| $cb.fetch_raw(&$target, mode.is_target_open(), pop))
                    .filter(|candi| !candi.is_empty())
                {
                    cb_fb.replace(candi[0].clone());
                    s = s.pop_num(candi.len());
                    let fitness = candi
                        .iter()
                        .map(|(f, fb)| mh::Product::new(*f, fb.denormalize()))
                        .collect();
                    let pool = candi.into_iter().map(|(_, fb)| fb.buf).collect::<Vec<_>>();
                    s = s.pool_and_fitness(mh::ndarray::arr2(&pool), fitness);
                } else {
                    s = s.pop_num(pop);
                }
                Self::$syn(s, cb_fb)
            }};
        }
        match target {
            Target::P(target, cb) => impl_solve!(target, cb, FbSyn),
            Target::S(target, cb) => impl_solve!(target, cb, SFbSyn),
        }
    }

    pub(crate) fn solve(self) -> Result<io::Fb, mh::ndarray::ShapeError> {
        match self {
            Self::FbSyn(s, _) => Ok(io::Fb::Fb(s.solve()?.into_result())),
            Self::SFbSyn(s, _) => Ok(io::Fb::SFb(s.solve()?.into_result())),
        }
    }

    // TODO: Get result with `cb_fb`
    #[allow(dead_code)]
    pub(crate) fn solve_cb(self) -> Result<CbFb, mh::ndarray::ShapeError> {
        match self {
            Self::FbSyn(s, cb_fb) => Ok(CbFb::Fb(s.solve()?.into_result(), cb_fb)),
            Self::SFbSyn(s, cb_fb) => Ok(CbFb::SFb(s.solve()?.into_result(), cb_fb)),
        }
    }
}

pub(crate) enum CbFb {
    Fb(FourBar, Option<(f64, NormFourBar)>),
    SFb(SFourBar, Option<(f64, SNormFourBar)>),
}
