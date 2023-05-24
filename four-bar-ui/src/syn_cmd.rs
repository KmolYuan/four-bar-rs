use crate::io;
use eframe::egui::mutex;
use four_bar::{cb, mh, syn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

type ListItem = (&'static str, &'static str, fn() -> SynMethod);

macro_rules! impl_list {
    ($name:ident) => {{
        let s = Self::$name();
        (s.name(), s.abbr(), Self::$name)
    }};
}

macro_rules! impl_method {
    ($(fn $method:ident, $sym:ident, $name:literal, $full_name:literal, $link:literal)+) => {
        pub(crate) const LIST: &[ListItem] = &[$(impl_list!($method)),+];

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
    pub(crate) gen: u64,
    pub(crate) pop: usize,
    pub(crate) mode: syn::Mode,
}

impl Default for SynConfig {
    fn default() -> Self {
        Self { gen: 50, pop: 200, mode: syn::Mode::Closed }
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

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct Task {
    pub(crate) gen: u64,
    pub(crate) time: u64,
    pub(crate) conv: Vec<f64>,
}

pub(crate) enum Solver {
    FbSyn(mh::utility::SolverBuilder<'static, syn::FbSyn>),
    SFbSyn(mh::utility::SolverBuilder<'static, syn::SFbSyn>),
}

impl Solver {
    pub(crate) fn new(
        setting: SynMethod,
        target: Target,
        cfg: SynConfig,
    ) -> (Self, Arc<mutex::RwLock<(u64, Task)>>) {
        #[cfg(target_arch = "wasm32")]
        use instant::Instant;
        #[cfg(not(target_arch = "wasm32"))]
        use std::time::Instant;
        let SynConfig { gen, pop, mode } = cfg;
        let task = Task { gen, time: 0, conv: Vec::new() };
        let task = Arc::new(mutex::RwLock::new((0, task)));
        macro_rules! impl_solve {
            ($target:ident, $cb:ident, $syn:ident) => {{
                let mut s = setting.build_solver(syn::$syn::from_curve(&$target, mode));
                if let Some(candi) = matches!(mode, syn::Mode::Closed | syn::Mode::Open)
                    .then(|| $cb.fetch_raw(&$target, mode.is_target_open(), pop))
                    .filter(|candi| !candi.is_empty())
                {
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
                {
                    let task = task.clone();
                    s = s.task(move |ctx| ctx.gen >= task.read().1.gen);
                }
                let s = {
                    let task = task.clone();
                    let t0 = Instant::now();
                    Solver::$syn(s.callback(move |ctx| {
                        let (gen, task) = &mut *task.write();
                        task.conv.push(ctx.best_f.fitness());
                        *gen = ctx.gen;
                        task.time = t0.elapsed().as_secs();
                    }))
                };
                (s, task)
            }};
        }
        match target {
            Target::P(target, cb) => impl_solve!(target, cb, FbSyn),
            Target::S(target, cb) => impl_solve!(target, cb, SFbSyn),
        }
    }

    pub(crate) fn solve(self) -> Result<io::Fb, mh::ndarray::ShapeError> {
        match self {
            Self::FbSyn(s) => Ok(io::Fb::Fb(s.solve()?.into_result())),
            Self::SFbSyn(s) => Ok(io::Fb::SFb(s.solve()?.into_result())),
        }
    }
}
