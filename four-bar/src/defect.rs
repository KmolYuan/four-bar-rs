/// Circuit type with a defect notation.
#[derive(Default)]
pub enum DefectGuard<C> {
    /// Defect-free curve (Closed curve)
    Closed(Vec<C>),
    /// Circuit defect curve (Open curve)
    Open(Vec<C>),
    /// Empty curve (Invalid linkage)
    #[default]
    Empty,
}

impl<C> DefectGuard<C> {
    /// Return true if the circuit is closed curve.
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed(_))
    }

    /// Return true if the circuit is open curve.
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open(_))
    }

    /// Return true if the circuit is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Return the length of the circuit.
    pub fn len(&self) -> usize {
        match self {
            Self::Closed(c) | Self::Open(c) => c.len(),
            Self::Empty => 0,
        }
    }

    /// Turn into `Option` type with closed curve check.
    ///
    /// Allow only closed circuit.
    pub fn to_closed(self) -> Option<Vec<C>> {
        match self {
            Self::Closed(c) => Some(c),
            _ => None,
        }
    }

    /// Turn into `Option` type with open curve check.
    ///
    /// Allow only open circuit.
    pub fn to_open(self) -> Option<Vec<C>> {
        match self {
            Self::Open(c) => Some(c),
            _ => None,
        }
    }

    /// Turn into `Option` type with empty check.
    ///
    /// Allow any circuit.
    pub fn to_circuit(self) -> Option<Vec<C>> {
        match self {
            Self::Closed(c) | Self::Open(c) => Some(c),
            Self::Empty => None,
        }
    }

    /// Turn into `Option` type with defect check.
    ///
    /// Allow closed and open circuit.
    pub fn to_defect_free(self) -> Option<Vec<C>> {
        match self {
            Self::Closed(c) | Self::Open(c) => Some(c),
            _ => None,
        }
    }
}
