use four_bar::mh::{De, Fa, Pso, Rga, Tlbo};
use serde::{Deserialize, Serialize};

macro_rules! impl_new {
    ($(fn $method:ident, $name:ident)+) => {$(
        pub(crate) const fn $method() -> Self {
            Self::$name($name::new())
        }
   )+};
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

impl std::fmt::Debug for SynCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl SynCmd {
    pub(crate) const fn name(&self) -> &'static str {
        match self {
            Self::De(_) => "Differential Evolution",
            Self::Fa(_) => "Firefly Algorithm",
            Self::Pso(_) => "Particle Swarm Optimization",
            Self::Rga(_) => "Real-coded Genetic Algorithm",
            Self::Tlbo(_) => "Teaching Learning Based Optimization",
        }
    }

    pub(crate) const fn abbr(&self) -> &'static str {
        match self {
            Self::De(_) => "DE",
            Self::Fa(_) => "FA",
            Self::Pso(_) => "PSO",
            Self::Rga(_) => "RGA",
            Self::Tlbo(_) => "TLBO",
        }
    }

    pub(crate) const fn link(&self) -> &'static str {
        match self {
            Self::De(_) => "https://en.wikipedia.org/wiki/Differential_evolution",
            Self::Fa(_) => "https://en.wikipedia.org/wiki/Firefly_algorithm",
            Self::Pso(_) => "https://en.wikipedia.org/wiki/Particle_swarm_optimization",
            Self::Rga(_) => "https://en.wikipedia.org/wiki/Genetic_algorithm",
            Self::Tlbo(_) => "https://doi.org/10.1016/j.cad.2010.12.015",
        }
    }

    impl_new! {
        fn de, De
        fn fa, Fa
        fn pso, Pso
        fn rga, Rga
        fn tlbo, Tlbo
    }
}
