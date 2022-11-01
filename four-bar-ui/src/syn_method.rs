use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Clone, Copy, PartialEq, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(clap::ValueEnum))]
pub(crate) enum SynMethod {
    #[default]
    De,
    Fa,
    Pso,
    Rga,
    Tlbo,
}

impl SynMethod {
    pub(crate) const LIST: &[Self] = &[Self::De, Self::Fa, Self::Pso, Self::Rga, Self::Tlbo];

    pub(crate) const fn name(&self) -> &'static str {
        match self {
            Self::De => "Differential Evolution",
            Self::Fa => "Firefly Algorithm",
            Self::Pso => "Particle Swarm Optimization",
            Self::Rga => "Real-coded Genetic Algorithm",
            Self::Tlbo => "Teaching Learning Based Optimization",
        }
    }

    pub(crate) const fn abbr(&self) -> &'static str {
        match self {
            Self::De => "DE",
            Self::Fa => "FA",
            Self::Pso => "PSO",
            Self::Rga => "RGA",
            Self::Tlbo => "TLBO",
        }
    }

    pub(crate) const fn link(&self) -> &'static str {
        match self {
            SynMethod::De => "https://en.wikipedia.org/wiki/Differential_evolution",
            SynMethod::Fa => "https://en.wikipedia.org/wiki/Firefly_algorithm",
            SynMethod::Pso => "https://en.wikipedia.org/wiki/Particle_swarm_optimization",
            SynMethod::Rga => "https://en.wikipedia.org/wiki/Genetic_algorithm",
            SynMethod::Tlbo => "https://doi.org/10.1016/j.cad.2010.12.015",
        }
    }
}
