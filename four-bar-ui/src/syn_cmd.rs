use four_bar::mh::{De, Fa, Pso, Rga, Tlbo};
use serde::{Deserialize, Serialize};

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

        $(pub(crate) const fn $method() -> Self { Self::$sym($sym::new()) })+

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
    De(De),
    Fa(Fa),
    Pso(Pso),
    Rga(Rga),
    Tlbo(Tlbo),
}

impl Default for SynCmd {
    fn default() -> Self {
        Self::De(De::default())
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
