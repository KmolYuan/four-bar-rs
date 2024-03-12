use std::f64::consts::TAU;

/// State of the linkage.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Default, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "lowercase")
)]
pub enum Stat {
    /// Circuit 1-1
    #[default]
    #[cfg_attr(feature = "serde", serde(alias = "C1B1"))]
    C1B1 = 1,
    /// Circuit 1-2
    #[cfg_attr(feature = "serde", serde(alias = "C1B2"))]
    C1B2 = 2,
    /// Circuit 2-1
    #[cfg_attr(feature = "serde", serde(alias = "C2B1"))]
    C2B1 = 3,
    /// Circuit 2-2
    #[cfg_attr(feature = "serde", serde(alias = "C2B2"))]
    C2B2 = 4,
}

impl std::fmt::Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::C1B1 => write!(f, "Circuit 1-1"),
            Self::C1B2 => write!(f, "Circuit 1-2"),
            Self::C2B1 => write!(f, "Circuit 2-1"),
            Self::C2B2 => write!(f, "Circuit 2-2"),
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
    pub const fn name_lowercase(&self) -> &'static str {
        match self {
            Self::C1B1 => "c1b1",
            Self::C1B2 => "c1b2",
            Self::C2B1 => "c2b1",
            Self::C2B2 => "c2b2",
        }
    }

    /// Get the uppercase name.
    pub const fn name_uppercase(&self) -> &'static str {
        match self {
            Self::C1B1 => "C1B1",
            Self::C1B2 => "C1B2",
            Self::C2B1 => "C2B1",
            Self::C2B2 => "C2B2",
        }
    }

    /// Check if the state is on circuit 1.
    pub fn is_c1(&self) -> bool {
        matches!(self, Self::C1B1 | Self::C1B2)
    }

    /// Check if the state is on branch 1.
    pub fn is_b1(&self) -> bool {
        matches!(self, Self::C1B1 | Self::C2B1)
    }

    /// Switch the circuit.
    pub fn switch_circuit(&mut self) {
        *self = match self {
            Self::C1B1 => Self::C2B1,
            Self::C1B2 => Self::C2B2,
            Self::C2B1 => Self::C1B1,
            Self::C2B2 => Self::C1B2,
        };
    }

    /// Switch the branch.
    pub fn switch_branch(&mut self) {
        *self = match self {
            Self::C1B1 => Self::C1B2,
            Self::C1B2 => Self::C1B1,
            Self::C2B1 => Self::C2B2,
            Self::C2B2 => Self::C2B1,
        };
    }
}

/// Angle boundary types. The input angle range.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum AngleBound {
    /// Closed curve
    Closed,
    /// Open curve with 1 circuits 2 branches (`[start, end]`)
    OpenC1B2([f64; 2]),
    /// Open curve with 2 circuits 2 branches (`[start, end]`)
    OpenC2B2([f64; 2]),
    /// Invalid
    #[default]
    Invalid,
}

impl AngleBound {
    /// The minimum input angle bound. (π/2)
    pub const MIN_ANGLE: f64 = std::f64::consts::FRAC_PI_2;

    /// Name of the angle bound.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Closed => "Closed curve",
            Self::OpenC1B2(_) => "Open curve with 1 circuits 2 branches",
            Self::OpenC2B2(_) => "Open curve with 2 circuits 2 branches",
            Self::Invalid => "Invalid",
        }
    }

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
                Self::OpenC1B2([d.acos(), TAU - d.acos()])
            }
            (false, true) => {
                let l33 = l3 + l4;
                let d = (l1 * l1 + l2 * l2 - l33 * l33) / (2. * l1 * l2);
                Self::OpenC1B2([-d.acos(), d.acos()])
            }
            (false, false) => {
                let numerator = l1 * l1 + l2 * l2;
                let denominator = 2. * l1 * l2;
                let l33 = l3 - l4;
                let d1 = (numerator - l33 * l33) / denominator;
                let l33 = l3 + l4;
                let d2 = (numerator - l33 * l33) / denominator;
                if stat.is_c1() {
                    Self::OpenC2B2([d1.acos(), d2.acos()])
                } else {
                    Self::OpenC2B2([TAU - d2.acos(), TAU - d1.acos()])
                }
            }
        }
    }

    /// Create a open and its reverse angle bound.
    pub fn open_and_rev_at(a: f64, b: f64) -> [Self; 2] {
        // No matter the type of the angle bound `OpenC1B2` or `OpenC2B2`
        [Self::OpenC1B2([a, b]), Self::OpenC1B2([b, a])]
    }

    /// Check the state is the same to the provided mode.
    pub fn check_mode(self, is_open: bool) -> Self {
        if self.is_valid() && self.is_open() == is_open {
            self
        } else {
            Self::Invalid
        }
    }

    /// Angle range must greater than [`AngleBound::MIN_ANGLE`].
    pub fn check_min(self) -> Self {
        match self {
            Self::OpenC1B2([a, b]) | Self::OpenC2B2([a, b]) => {
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
            Self::OpenC1B2(a) | Self::OpenC2B2(a) => Some(a),
            Self::Invalid => None,
        }
    }

    /// Return true if the bounds is open.
    pub fn is_open(&self) -> bool {
        !matches!(self, Self::Closed | Self::Invalid)
    }

    /// Check if the data is valid.
    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::Invalid)
    }

    /// List all states.
    pub fn get_states(&self) -> Vec<Stat> {
        match self {
            Self::Closed => vec![Stat::C1B1, Stat::C2B1],
            Self::OpenC1B2(_) => vec![Stat::C1B1, Stat::C1B2],
            Self::OpenC2B2(_) => vec![Stat::C1B1, Stat::C1B2, Stat::C2B1, Stat::C2B2],
            Self::Invalid => vec![Stat::C1B1],
        }
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
pub trait Statable: PlanarLoop + Clone {
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

    /// Return the type of this linkage.
    fn ty(&self) -> FourBarTy {
        FourBarTy::from_loop(self.planar_loop())
    }

    /// Input angle bounds of the linkage.
    fn angle_bound(&self) -> AngleBound {
        let stat = self.stat();
        AngleBound::from_planar_loop(self.planar_loop(), stat)
    }

    /// Return `true` will actives the inversion.
    fn inv(&self) -> bool {
        !self.stat().is_c1()
    }

    /// List all states from a linkage.
    fn to_states(self) -> Vec<Self> {
        let bound = self.angle_bound();
        self.states_from_bound(bound)
    }

    /// List all states except the current state from a linkage.
    fn other_states(&self) -> Vec<Self> {
        self.other_states_from_bound(self.angle_bound())
    }

    /// List all states from a calculated bound.
    fn states_from_bound(self, bound: AngleBound) -> Vec<Self> {
        let mut states = self.other_states_from_bound(bound);
        states.push(self);
        states
    }

    /// List all states except the current state from a calculated bound.
    fn other_states_from_bound(&self, bound: AngleBound) -> Vec<Self> {
        let stat = self.stat();
        bound
            .get_states()
            .into_iter()
            .filter(|s| *s != stat)
            .map(|s| self.clone().with_stat(s))
            .collect()
    }
}

impl<S> Statable for S
where
    S: std::ops::DerefMut + PlanarLoop + Clone,
    S::Target: Statable,
{
    fn stat_mut(&mut self) -> &mut Stat {
        self.deref_mut().stat_mut()
    }

    fn stat(&self) -> Stat {
        self.deref().stat()
    }
}

/// Planar loop of the linkage.
pub trait PlanarLoop {
    /// Get the planar loop.
    fn planar_loop(&self) -> [f64; 4];
    /// Set the link lengths as the planar loop.
    fn set_to_planar_loop(&mut self) {}
}
