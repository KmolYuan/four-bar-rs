use super::PlanarLoop;

/// State of the linkage.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged, rename_all = "lowercase"))]
pub enum Stat {
    /// Circuit 1, branch 1
    #[default]
    C1B1 = 1,
    /// Circuit 1, branch 2
    C1B2 = 2,
    /// Circuit 2, branch 1
    C2B1 = 3,
    /// Circuit 2, branch 2
    C2B2 = 4,
}

impl std::fmt::Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::C1B1 => write!(f, "Circuit 1, branch 1"),
            Self::C1B2 => write!(f, "Circuit 1, branch 2"),
            Self::C2B1 => write!(f, "Circuit 2, branch 1"),
            Self::C2B2 => write!(f, "Circuit 2, branch 2"),
        }
    }
}

/// Error for state conversion.
#[derive(Debug)]
pub struct StatError;

impl std::fmt::Display for StatError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid state")
    }
}

impl std::error::Error for StatError {}

impl TryFrom<u8> for Stat {
    type Error = StatError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::C1B1),
            2 => Ok(Self::C1B2),
            3 => Ok(Self::C2B1),
            4 => Ok(Self::C2B2),
            _ => Err(StatError),
        }
    }
}

impl Stat {
    /// List for two circuits.
    pub fn list2() -> Vec<Self> {
        vec![Self::C1B1, Self::C2B1]
    }

    /// List for two circuits, two branches.
    pub fn list4() -> Vec<Self> {
        vec![Self::C1B1, Self::C1B2, Self::C2B1, Self::C2B2]
    }

    /// Check if the state is on circuit 1.
    pub fn is_c1(&self) -> bool {
        matches!(self, Self::C1B1 | Self::C1B2)
    }

    /// Check if the state is on branch 1.
    pub fn is_b1(&self) -> bool {
        matches!(self, Self::C1B1 | Self::C2B1)
    }

    /// List for other states for two circuits.
    pub fn list2_others(&self) -> Vec<Self> {
        self.list_others(Self::list2(), 2)
    }

    /// List for other states for two circuits, two branches.
    pub fn list4_others(&self) -> Vec<Self> {
        self.list_others(Self::list4(), 4)
    }

    fn list_others(&self, list: Vec<Self>, cap: usize) -> Vec<Self> {
        let mut set = std::collections::HashSet::with_capacity(cap);
        set.extend(list);
        set.remove(self);
        Vec::from_iter(set)
    }
}

/// State of the linkage.
pub trait Statable: Clone {
    /// Get the state mutable reference.
    fn stat_mut(&mut self) -> &mut Stat;
    /// Get the state.
    fn stat(&self) -> Stat;

    /// Set the state.
    fn set_stat(&mut self, stat: Stat) {
        *self.stat_mut() = stat;
    }

    /// Build with state.
    fn with_stat(mut self, stat: Stat) -> Self {
        self.set_stat(stat);
        self
    }

    /// Get the inversion state.
    fn inv(&self) -> bool
    where
        Self: PlanarLoop,
    {
        let stat = self.stat();
        if self.has_branch() {
            !stat.is_b1()
        } else {
            !stat.is_c1()
        }
    }

    /// Get all states from a linkage.
    fn get_states(self) -> Vec<Self>
    where
        Self: PlanarLoop,
    {
        let stat = self.stat();
        let list = if self.has_branch() {
            stat.list4_others()
        } else {
            stat.list2_others()
        };
        let mut list = list
            .into_iter()
            .map(|stat| self.clone().with_stat(stat))
            .collect::<Vec<_>>();
        list.push(self);
        list
    }
}

impl<S> Statable for S
where
    S: std::ops::DerefMut + Clone,
    S::Target: Statable,
{
    fn stat_mut(&mut self) -> &mut Stat {
        self.deref_mut().stat_mut()
    }

    fn stat(&self) -> Stat {
        self.deref().stat()
    }
}
