use crate::io;
use eframe::egui::mutex;
use four_bar::{cb, mh, syn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

type ListItem = (&'static str, &'static str, fn() -> SynCmd);

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
pub(crate) enum SynCmd {
    De(mh::De),
    Fa(mh::Fa),
    Pso(mh::Pso),
    Rga(mh::Rga),
    Tlbo(mh::Tlbo),
}

impl Default for SynCmd {
    fn default() -> Self {
        Self::De(mh::De::default())
    }
}

impl SynCmd {
    impl_method! {
        fn de, De, "DE", "Differential Evolution", "https://en.wikipedia.org/wiki/Differential_evolution"
        fn fa, Fa, "FA", "Firefly Algorithm", "https://en.wikipedia.org/wiki/Firefly_algorithm"
        fn pso, Pso, "PSO", "Particle Swarm Optimization", "https://en.wikipedia.org/wiki/Particle_swarm_optimization"
        fn rga, Rga, "RGA", "Real-coded Genetic Algorithm", "https://en.wikipedia.org/wiki/Genetic_algorithm"
        fn tlbo, Tlbo, "TLBO", "Teaching Learning Based Optimization", "https://doi.org/10.1016/j.cad.2010.12.015"
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

pub(crate) struct SynSolver<S: mh::Setting> {
    setting: S,
    target: Target,
    pop: usize,
    mode: syn::Mode,
    task: Arc<mutex::RwLock<(u64, Task)>>,
}

impl<S: mh::Setting> SynSolver<S> {
    pub(crate) fn new(
        setting: S,
        target: Target,
        pop: usize,
        mode: syn::Mode,
        task: Arc<mutex::RwLock<(u64, Task)>>,
    ) -> Self {
        Self { setting, target, pop, mode, task }
    }

    pub(crate) fn solve(self) -> io::Fb {
        #[cfg(target_arch = "wasm32")]
        use instant::Instant;
        #[cfg(not(target_arch = "wasm32"))]
        use std::time::Instant;
        let t0 = Instant::now();
        let Self { setting, target, pop, mode, task } = self;
        macro_rules! impl_solve {
            ($target:ident, $cb:ident, $fb:ident, $syn:ident) => {{
                let mut s =
                    four_bar::mh::Solver::build(setting, syn::$syn::from_curve(&$target, mode));
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
                let fb = s
                    .task(|ctx| ctx.gen >= task.read().1.gen)
                    .callback(|ctx| {
                        let (gen, task) = &mut *task.write();
                        task.conv.push(ctx.best_f.fitness());
                        *gen = ctx.gen;
                        task.time = t0.elapsed().as_secs();
                    })
                    .solve()
                    .unwrap()
                    .into_result();
                io::Fb::$fb(fb)
            }};
        }
        match target {
            Target::P(target, cb) => impl_solve!(target, cb, Fb, FbSyn),
            Target::S(target, cb) => impl_solve!(target, cb, SFb, SFbSyn),
        }
    }
}
