use crate::io;
use four_bar::*;
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
pub(crate) struct SynCfg {
    /// Fix the seed to get a determined result, default to random
    #[cfg_attr(not(target_arch = "wasm32"), clap(short, long))]
    pub(crate) seed: Option<u64>,
    /// Number of generation
    #[cfg_attr(not(target_arch = "wasm32"), clap(short, long, default_value_t = 50))]
    pub(crate) gen: u64,
    /// Number of population (the fetch number in atlas)
    #[cfg_attr(not(target_arch = "wasm32"), clap(short, long, default_value_t = 200))]
    pub(crate) pop: usize,
    /// Number of the points (resolution) in curve production
    #[cfg_attr(not(target_arch = "wasm32"), clap(long, default_value_t = 180))]
    pub(crate) res: usize,
    /// Specify the mechanism on the origin and unit scale
    #[cfg_attr(not(target_arch = "wasm32"), clap(long))]
    pub(crate) on_unit: bool,
    #[cfg_attr(not(target_arch = "wasm32"), clap(skip = syn::Mode::Closed))]
    pub(crate) mode: syn::Mode,
}

impl Default for SynCfg {
    fn default() -> Self {
        Self {
            seed: None,
            gen: 50,
            pop: 200,
            res: 180,
            on_unit: false,
            mode: syn::Mode::Closed,
        }
    }
}

#[derive(Clone)]
pub(crate) enum Target<'a> {
    P(Cow<'a, [[f64; 2]]>, Cow<'a, atlas::FbAtlas>),
    S(Cow<'a, [[f64; 3]]>, Cow<'a, atlas::SFbAtlas>),
}

type FbBuilder<'a> = mh::utility::SolverBuilder<'a, syn::FbSyn>;
type SFbBuilder<'a> = mh::utility::SolverBuilder<'a, syn::SFbSyn>;

pub(crate) enum Solver<'a> {
    FbSyn(FbBuilder<'a>, Option<(f64, NormFourBar)>),
    SFbSyn(SFbBuilder<'a>, Option<(f64, SNormFourBar)>),
}

impl<'a> Solver<'a> {
    pub(crate) fn new<S, C>(
        alg: SynAlg,
        target: Target,
        cfg: SynCfg,
        stop: S,
        mut callback: C,
    ) -> Self
    where
        S: Fn() -> bool + Send + 'a,
        C: FnMut(f64, u64) + Send + 'a,
    {
        let SynCfg { seed, gen, pop, mode, res, on_unit } = cfg;
        macro_rules! impl_solve {
            ($target:ident, $atlas:ident, $syn:ident) => {{
                let mut syn = syn::$syn::from_curve(&$target, mode).res(res);
                if on_unit {
                    syn = syn.on_unit();
                }
                let mut s = alg
                    .build_solver(syn)
                    .seed(seed)
                    .task(move |ctx| !stop() && ctx.gen >= gen)
                    .callback(move |ctx| callback(ctx.best_f.fitness(), ctx.gen));
                let mut atlas_fb = None;
                if let Some(candi) = matches!(mode, syn::Mode::Closed | syn::Mode::Open)
                    .then(|| $atlas.fetch_raw(&$target, mode.is_target_open(), pop))
                    .filter(|candi| !candi.is_empty())
                {
                    use four_bar::mech::{IntoVectorized as _, Normalized as _};
                    use mh::ndarray::Array2;
                    atlas_fb = Some(candi[0].clone());
                    let pop = candi.len();
                    s = s.pop_num(pop);
                    let fitness = candi
                        .iter()
                        .map(|(f, fb)| mh::Product::new(*f, fb.clone().denormalize()))
                        .collect();
                    let pool = candi
                        .into_iter()
                        .flat_map(|(_, fb)| fb.into_vectorized().0)
                        .collect::<Vec<_>>();
                    let pool = Array2::from_shape_vec((pop, pool.len() / pop), pool).unwrap();
                    s = s.pool_and_fitness(pool, fitness);
                } else {
                    s = s.pop_num(pop);
                }
                Self::$syn(s, atlas_fb)
            }};
        }
        match target {
            Target::P(target, atlas) => impl_solve!(target, atlas, FbSyn),
            Target::S(target, atlas) => impl_solve!(target, atlas, SFbSyn),
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
            ($syn:ident, $s:ident, $atlas_fb:ident) => {{
                let s = $s.solve()?;
                let h = s.func().harmonic();
                let (err, fb) = s.into_err_result();
                Ok((err, h, SolvedFb::$syn(fb, $atlas_fb)))
            }};
        }
        match self {
            Self::FbSyn(s, atlas_fb) => impl_solve!(Fb, s, atlas_fb),
            Self::SFbSyn(s, atlas_fb) => impl_solve!(SFb, s, atlas_fb),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) enum SolvedFb {
    Fb(FourBar, Option<(f64, NormFourBar)>),
    SFb(SFourBar, Option<(f64, SNormFourBar)>),
}
