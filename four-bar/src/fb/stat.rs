use std::f64::consts::TAU;

/// State of the linkage.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Stat {
    /// Circuit 1, branch 1
    #[default]
    #[cfg_attr(feature = "serde", serde(alias = "c1b1"))]
    C1B1 = 1,
    /// Circuit 1, branch 2
    #[cfg_attr(feature = "serde", serde(alias = "c1b2"))]
    C1B2 = 2,
    /// Circuit 2, branch 1
    #[cfg_attr(feature = "serde", serde(alias = "c2b1"))]
    C2B1 = 3,
    /// Circuit 2, branch 2
    #[cfg_attr(feature = "serde", serde(alias = "c2b2"))]
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
    /// Get the lowercase name.
    pub fn name_lowercase(&self) -> &'static str {
        match self {
            Self::C1B1 => "c1b1",
            Self::C1B2 => "c1b2",
            Self::C2B1 => "c2b1",
            Self::C2B2 => "c2b2",
        }
    }

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

/// Angle boundary types. The input angle range.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum AngleBound {
    /// Closed curve
    Closed,
    /// Open curve (`[start, end]`)
    Open([f64; 2]),
    /// Open curve with branch (`[[start, end]; 2]`)
    OpenBranch([f64; 2]),
    /// Invalid
    #[default]
    Invalid,
}

impl AngleBound {
    /// The minimum input angle bound. (π/2)
    pub const MIN_ANGLE: f64 = std::f64::consts::FRAC_PI_2;

    /// Check angle bound from a planar loop.
    pub fn from_planar_loop(mut planar_loop: [f64; 4], stat: Stat) -> Self {
        let [l1, l2, l3, l4] = planar_loop;
        planar_loop.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        if planar_loop[3] > planar_loop[..3].iter().sum() {
            return Self::Invalid;
        }
        match (l1 + l2 <= l3 + l4, (l1 - l2).abs() >= (l3 - l4).abs()) {
            (true, true) => Self::Closed,
            (true, false) => {
                let l33 = l3 - l4;
                let d = (l1 * l1 + l2 * l2 - l33 * l33) / (2. * l1 * l2);
                Self::Open([d.acos(), TAU - d.acos()])
            }
            (false, true) => {
                let l33 = l3 + l4;
                let d = (l1 * l1 + l2 * l2 - l33 * l33) / (2. * l1 * l2);
                Self::Open([-d.acos(), d.acos()])
            }
            (false, false) => {
                let numerator = l1 * l1 + l2 * l2;
                let denominator = 2. * l1 * l2;
                let l33 = l3 - l4;
                let d1 = (numerator - l33 * l33) / denominator;
                let l33 = l3 + l4;
                let d2 = (numerator - l33 * l33) / denominator;
                if stat.is_c1() {
                    Self::OpenBranch([d1.acos(), d2.acos()])
                } else {
                    Self::OpenBranch([TAU - d2.acos(), TAU - d1.acos()])
                }
            }
        }
    }

    /// Check there has two branches.
    pub fn has_branch(&self) -> bool {
        matches!(self, Self::OpenBranch(_))
    }

    /// Create a open and its reverse angle bound.
    pub fn open_and_rev_at(a: f64, b: f64) -> [Self; 2] {
        [Self::Open([a, b]), Self::Open([b, a])]
    }

    /// Check the state is the same to the provided mode.
    pub fn check_mode(self, is_open: bool) -> Self {
        match (&self, is_open) {
            (Self::Closed, false) | (Self::Open(_), true) => self,
            _ => Self::Invalid,
        }
    }

    /// Angle range must greater than [`AngleBound::MIN_ANGLE`].
    pub fn check_min(self) -> Self {
        match self {
            Self::Open([a, b]) | Self::OpenBranch([a, b]) => {
                let b = if b > a { b } else { b + TAU };
                if b - a > Self::MIN_ANGLE {
                    self
                } else {
                    Self::Invalid
                }
            }
            _ => self,
        }
    }

    /// Turn into boundary values.
    pub fn to_value(self) -> Option<[f64; 2]> {
        match self {
            Self::Closed => Some([0., TAU]),
            Self::Open(a) | Self::OpenBranch(a) => Some(a),
            Self::Invalid => None,
        }
    }

    /// Check if the data is valid.
    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::Invalid)
    }
}

/// Type of the four-bar linkage.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum FourBarTy {
    /// Grashof double crank (Drag-link)
    GCCC,
    /// Grashof crank rocker
    GCRR,
    /// Grashof double rocker
    GRCR,
    /// Grashof rocker crank
    GRRC,
    /// Non-Grashof triple rocker (ground link is the longest)
    RRR1,
    /// Non-Grashof triple rocker (driver link is the longest)
    RRR2,
    /// Non-Grashof triple rocker (coupler link is the longest)
    RRR3,
    /// Non-Grashof triple rocker (follower link is the longest)
    RRR4,
    /// Invalid
    Invalid,
}

impl FourBarTy {
    /// Detect from four-bar loop `[l1, l2, l3, l4]`.
    pub fn from_loop(mut fb_loop: [f64; 4]) -> Self {
        let [l1, l2, l3, l4] = fb_loop;
        fb_loop.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        let [s, p, q, l] = fb_loop;
        if l > s + p + q {
            return Self::Invalid;
        }
        macro_rules! arms {
            ($d:expr, $c1:expr, $c2:expr, $c3:expr, $c4:expr) => {
                match $d {
                    d if d == l1 => $c1,
                    d if d == l2 => $c2,
                    d if d == l3 => $c3,
                    d if d == l4 => $c4,
                    _ => unreachable!(),
                }
            };
        }
        if s + l < p + q {
            arms!(s, Self::GCCC, Self::GCRR, Self::GRCR, Self::GRRC)
        } else {
            arms!(l, Self::RRR1, Self::RRR2, Self::RRR3, Self::RRR4)
        }
    }

    /// Name of the type.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::GCCC => "Grashof double crank (Drag-link, GCCC)",
            Self::GCRR => "Grashof crank rocker (GCRR)",
            Self::GRCR => "Grashof double rocker (GRCR)",
            Self::GRRC => "Grashof rocker crank (GRRC)",
            Self::RRR1 => "Non-Grashof triple rocker (RRR1)",
            Self::RRR2 => "Non-Grashof triple rocker (RRR2)",
            Self::RRR3 => "Non-Grashof triple rocker (RRR3)",
            Self::RRR4 => "Non-Grashof triple rocker (RRR4)",
            Self::Invalid => "Invalid",
        }
    }

    /// Check if the type is valid.
    pub const fn is_valid(&self) -> bool {
        !matches!(self, Self::Invalid)
    }

    /// Return true if the type is Grashof linkage.
    pub const fn is_grashof(&self) -> bool {
        matches!(self, Self::GCCC | Self::GCRR | Self::GRCR | Self::GRRC)
    }

    /// Return true if the type has continuous motion.
    pub const fn is_closed_curve(&self) -> bool {
        matches!(self, Self::GCCC | Self::GCRR)
    }

    /// Return true if the type has non-continuous motion.
    pub const fn is_open_curve(&self) -> bool {
        matches!(
            self,
            Self::GRCR | Self::GRRC | Self::RRR1 | Self::RRR2 | Self::RRR3 | Self::RRR4
        )
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

// TODO: Merge with `Statable` trait.
/// Planar loop of the linkage.
pub trait PlanarLoop {
    /// Get the planar loop.
    fn planar_loop(&self) -> [f64; 4];

    /// Return the type of this linkage.
    fn ty(&self) -> FourBarTy {
        FourBarTy::from_loop(self.planar_loop())
    }

    /// Input angle bounds of the linkage.
    fn angle_bound(&self) -> AngleBound
    where
        Self: Statable,
    {
        let stat = self.stat();
        AngleBound::from_planar_loop(self.planar_loop(), stat)
    }

    /// Check if the range of motion has two branches.
    fn has_branch(&self) -> bool {
        let mut planar_loop @ [l1, l2, l3, l4] = self.planar_loop();
        planar_loop.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        planar_loop[3] < planar_loop[..3].iter().sum()
            && l1 + l2 > l3 + l4
            && (l1 - l2).abs() < (l3 - l4).abs()
    }
}
