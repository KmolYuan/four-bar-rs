use crate::io;
use four_bar::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

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
#[cfg_attr(not(target_arch = "wasm32"), derive(clap::Args))]
pub(crate) struct SynConfig {
    /// Fix the seed to get a determined result, default to random
    #[cfg_attr(not(target_arch = "wasm32"), clap(short, long))]
    pub(crate) seed: Option<u64>,
    /// Number of generation
    #[cfg_attr(not(target_arch = "wasm32"), clap(short, long, default_value_t = 50))]
    pub(crate) gen: u64,
    /// Number of population (the fetch number in codebook)
    #[cfg_attr(not(target_arch = "wasm32"), clap(short, long, default_value_t = 200))]
    pub(crate) pop: usize,
    /// Number of the points (resolution) in curve production
    #[cfg_attr(not(target_arch = "wasm32"), clap(long, default_value_t = 180))]
    pub(crate) res: usize,
    #[cfg_attr(not(target_arch = "wasm32"), clap(skip = syn::Mode::Closed))]
    pub(crate) mode: syn::Mode,
}

impl Default for SynConfig {
    fn default() -> Self {
        Self {
            seed: None,
            gen: 50,
            pop: 200,
            res: 180,
            mode: syn::Mode::Closed,
        }
    }
}

#[derive(Clone)]
pub(crate) enum Target<'a> {
    P(Cow<'a, [[f64; 2]]>, Cow<'a, cb::FbCodebook>),
    S(Cow<'a, [[f64; 3]]>, Cow<'a, cb::SFbCodebook>),
}

type FbBuilder<'a> = mh::utility::SolverBuilder<'a, syn::FbSyn>;
type SFbBuilder<'a> = mh::utility::SolverBuilder<'a, syn::SFbSyn>;

pub(crate) enum Solver<'a> {
    FbSyn(FbBuilder<'a>, Option<(f64, NormFourBar)>),
    SFbSyn(SFbBuilder<'a>, Option<(f64, SNormFourBar)>),
}

impl<'a> Solver<'a> {
    pub(crate) fn new<C>(method: SynMethod, target: Target, cfg: SynConfig, mut f: C) -> Self
    where
        C: FnMut(f64, u64) + Send + 'a,
    {
        let SynConfig { seed, gen, pop, mode, res } = cfg;
        macro_rules! impl_solve {
            ($target:ident, $cb:ident, $syn:ident) => {{
                let mut s = method
                    .build_solver(syn::$syn::from_curve(&$target, mode).res(res))
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

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn solve_verbose(self) -> Result<(f64, usize, SolvedFb), mh::ndarray::ShapeError> {
        macro_rules! impl_solve {
            ($syn:ident, $s:ident, $cb_fb:ident) => {{
                let s = $s.solve()?;
                let h = s.func().harmonic();
                let (err, fb) = s.into_err_result();
                Ok((err, h, SolvedFb::$syn(fb, $cb_fb)))
            }};
        }
        match self {
            Self::FbSyn(s, cb_fb) => impl_solve!(Fb, s, cb_fb),
            Self::SFbSyn(s, cb_fb) => impl_solve!(SFb, s, cb_fb),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) enum SolvedFb {
    Fb(FourBar, Option<(f64, NormFourBar)>),
    SFb(SFourBar, Option<(f64, SNormFourBar)>),
}
