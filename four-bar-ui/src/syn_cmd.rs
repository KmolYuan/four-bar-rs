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
    /// Use the distance-discrepancy method
    #[cfg_attr(not(target_arch = "wasm32"), clap(long = "dd"))]
    pub(crate) use_dd: bool,
    #[cfg_attr(not(target_arch = "wasm32"), clap(skip = CFG_DEF.mode))]
    pub(crate) mode: syn::Mode,
}

const CFG_DEF: SynCfg = SynCfg {
    seed: None,
    gen: 50,
    pop: 200,
    res: 180,
    on_unit: false,
    use_dd: false,
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
        tar_curve: Cow<'a, [[f64; 2]]>,
        tar_fb: Option<FourBar>,
        atlas: Option<&'b atlas::FbAtlas>,
    },
    MFb {
        target: Cow<'a, [([f64; 2], [f64; 2])]>,
        tar_fb: Option<MFourBar>,
    },
    SFb {
        tar_curve: Cow<'a, [[f64; 3]]>,
        tar_fb: Option<SFourBar>,
        atlas: Option<&'b atlas::SFbAtlas>,
    },
}

impl<'a, 'b> Target<'a, 'b> {
    pub(crate) fn fb(
        tar_curve: Cow<'a, [[f64; 2]]>,
        tar_fb: Option<FourBar>,
        atlas: Option<&'b atlas::FbAtlas>,
    ) -> Self {
        Self::Fb { tar_curve, tar_fb, atlas }
    }

    pub(crate) fn mfb(target: Cow<'a, [([f64; 2], [f64; 2])]>, tar_fb: Option<MFourBar>) -> Self {
        Self::MFb { target, tar_fb }
    }

    pub(crate) fn sfb(
        tar_curve: Cow<'a, [[f64; 3]]>,
        tar_fb: Option<SFourBar>,
        atlas: Option<&'b atlas::SFbAtlas>,
    ) -> Self {
        Self::SFb { tar_curve, tar_fb, atlas }
    }
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) struct PSynData<'a, MDe, F: mh::ObjFunc, const D: usize> {
    pub(crate) s: SolverBox<'a, F>,
    pub(crate) tar_curve: Cow<'a, [[f64; D]]>,
    pub(crate) tar_fb: Option<MDe>,
    pub(crate) atlas_fb: Option<(f64, MDe)>,
}

impl<'a, MDe, F, const D: usize> PSynData<'a, MDe, F, D>
where
    F: mh::ObjFunc<Ys = mh::WithProduct<f64, MDe>>,
    MDe: Default + Clone + Sync + Send + 'static,
{
    fn new<M, S, C, const N: usize>(
        s: SolverBox<'a, F>,
        tar_curve: Cow<'a, [[f64; D]]>,
        tar_fb: Option<M::De>,
        atlas: Option<&atlas::Atlas<M, N, D>>,
        cfg: SynCfg,
        stop: S,
        mut callback: C,
    ) -> Self
    where
        M: atlas::Code<N, D> + mech::Normalized<D, De = MDe>,
        S: Fn() -> bool + Send + 'a,
        C: FnMut(f64, u64) + Send + 'a,
        efd::U<D>: efd::EfdDim<D>,
        efd::Efd<D>: Sync,
    {
        let mut s = s
            .seed(cfg.seed)
            .pop_num(cfg.pop)
            .task(move |ctx| !stop() && ctx.gen >= cfg.gen)
            .callback(move |ctx| callback(ctx.best.get_eval(), ctx.gen));
        // FIXME: Try block
        let atlas_fb = if let Some((fb, pool, pool_y)) = (|| {
            let atlas = atlas.filter(|_| !cfg.mode.is_partial())?;
            let (best, candi) = atlas.fetch_raw(&tar_curve, cfg.mode.is_target_open(), cfg.pop)?;
            let pool_y = candi
                .iter()
                .map(|(f, fb)| mh::WithProduct::new(*f, fb.clone().denormalize()))
                .collect();
            let pool = candi
                .into_iter()
                .map(|(_, fb)| fb.into_vectorized().0)
                .collect();
            Some((best, pool, pool_y))
        })() {
            s = s.init_pool(mh::Pool::Ready { pool, pool_y });
            Some(fb)
        } else {
            None
        };
        Self { s, tar_curve, tar_fb, atlas_fb }
    }

    fn solve(self) -> MDe {
        self.s.solve().into_result()
    }
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) struct MSynData<'a, Y, F: mh::ObjFunc>
where
    Y: mh::Fitness<Eval = f64>,
    F: mh::ObjFunc<Ys = mh::WithProduct<Y, MFourBar>>,
{
    pub(crate) s: SolverBox<'a, F>,
    pub(crate) tar_p: Vec<[f64; 2]>,
    pub(crate) tar_v: Vec<[f64; 2]>,
    pub(crate) tar_fb: Option<MFourBar>,
}

impl<'a, Y, F: mh::ObjFunc> MSynData<'a, Y, F>
where
    Y: mh::Fitness<Eval = f64>,
    F: mh::ObjFunc<Ys = mh::WithProduct<Y, MFourBar>>,
{
    fn new<S, C>(
        s: SolverBox<'a, F>,
        tar_curve: Vec<[f64; 2]>,
        tar_pose: Vec<[f64; 2]>,
        tar_fb: Option<MFourBar>,
        cfg: SynCfg,
        stop: S,
        mut callback: C,
    ) -> Self
    where
        S: Fn() -> bool + Send + 'a,
        C: FnMut(f64, u64) + Send + 'a,
    {
        use mh::pareto::Best as _;
        let s = s
            .seed(cfg.seed)
            .pop_num(cfg.pop)
            .task(move |ctx| !stop() && ctx.gen >= cfg.gen)
            .callback(move |ctx| callback(ctx.best.get_eval(), ctx.gen));
        Self { s, tar_p: tar_curve, tar_v: tar_pose, tar_fb }
    }

    fn solve(self) -> MFourBar {
        self.s.solve().into_result()
    }
}

pub(crate) enum Solver<'a> {
    Fb(PSynData<'a, FourBar, syn::FbSyn, 2>),
    MFb(MSynData<'a, syn::MOFit, syn::MFbSyn>),
    SFb(PSynData<'a, SFourBar, syn::SFbSyn, 3>),
    DDFb(PSynData<'a, FourBar, syn::FbDDSyn, 2>),
    DDSFb(PSynData<'a, SFourBar, syn::SFbDDSyn, 3>),
    DDMFb(MSynData<'a, f64, syn::MFbDDSyn>),
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
        macro_rules! build_solver {
            ($ty:ident, $tar_curve:ident) => {{
                let mut obj = syn::$ty::from_curve(&$tar_curve, cfg.mode).res(cfg.res);
                if cfg.on_unit {
                    obj = obj.on_unit();
                }
                alg.build_solver(obj)
            }};
            (@ $ty:ident, $target:ident) => {{
                let (tar_curve, tar_pose): (Vec<_>, Vec<_>) =
                    $target.into_owned().into_iter().unzip();
                let mut obj = syn::$ty::from_uvec(&tar_curve, &tar_pose, cfg.mode).res(cfg.res);
                if cfg.on_unit {
                    obj = obj.on_unit();
                }
                (alg.build_solver(obj), tar_curve, tar_pose)
            }};
        }
        match target {
            Target::Fb { tar_curve, tar_fb, atlas } => {
                if cfg.use_dd {
                    let s = build_solver!(FbDDSyn, tar_curve);
                    Self::DDFb(PSynData::new(
                        s, tar_curve, tar_fb, atlas, cfg, stop, callback,
                    ))
                } else {
                    let s = build_solver!(FbSyn, tar_curve);
                    Self::Fb(PSynData::new(
                        s, tar_curve, tar_fb, atlas, cfg, stop, callback,
                    ))
                }
            }
            Target::MFb { target, tar_fb } => {
                if cfg.use_dd {
                    let (s, tar_curve, tar_pose) = build_solver!(@MFbDDSyn, target);
                    Self::DDMFb(MSynData::new(
                        s, tar_curve, tar_pose, tar_fb, cfg, stop, callback,
                    ))
                } else {
                    let (s, tar_curve, tar_pose) = build_solver!(@MFbSyn, target);
                    Self::MFb(MSynData::new(
                        s, tar_curve, tar_pose, tar_fb, cfg, stop, callback,
                    ))
                }
            }
            Target::SFb { tar_curve, tar_fb, atlas } => {
                if cfg.use_dd {
                    let s = build_solver!(SFbDDSyn, tar_curve);
                    Self::DDSFb(PSynData::new(
                        s, tar_curve, tar_fb, atlas, cfg, stop, callback,
                    ))
                } else {
                    let s = build_solver!(SFbSyn, tar_curve);
                    Self::SFb(PSynData::new(
                        s, tar_curve, tar_fb, atlas, cfg, stop, callback,
                    ))
                }
            }
        }
    }

    pub(crate) fn solve(self) -> io::Fb {
        match self {
            Self::Fb(s) => io::Fb::P(s.solve()),
            Self::MFb(s) => io::Fb::M(s.solve()),
            Self::SFb(s) => io::Fb::S(s.solve()),
            Self::DDFb(s) => io::Fb::P(s.solve()),
            Self::DDSFb(s) => io::Fb::S(s.solve()),
            Self::DDMFb(s) => io::Fb::M(s.solve()),
        }
    }
}
