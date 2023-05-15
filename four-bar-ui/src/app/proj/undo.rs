use four_bar::FourBar;

pub(crate) trait Delta: Sized {
    type State: Clone;

    fn delta(a: &Self::State, b: &Self::State) -> Option<Self>;
    fn undo(&self, state: &mut Self::State);
    fn redo(&self, state: &mut Self::State);
    fn try_merge(&mut self, rhs: &Self) -> Option<()>;
}

macro_rules! impl_delta {
    ($ty_name:ident, $state:ident, $(($f:ident, $ty:ty, $m:ident, $m_mut:ident)),+ $(,)?) => {
        #[derive(PartialEq)]
        pub(crate) enum $ty_name {
            $($f($ty, $ty)),+
        }

        impl Delta for $ty_name {
            type State = $state;

            fn delta(a: &Self::State, b: &Self::State) -> Option<Self> {
                match (a, b) {
                    $(_ if a.$m() != b.$m() => Some(Self::$f(a.$m(), b.$m())),)+
                    _ => None,
                }
            }

            fn undo(&self, state: &mut Self::State) {
                match self { $(Self::$f(v, _) => *state.$m_mut() = *v,)+ }
            }

            fn redo(&self, state: &mut Self::State) {
                match self { $(Self::$f(_, v) => *state.$m_mut() = *v,)+ }
            }

            fn try_merge(&mut self, rhs: &Self) -> Option<()> {
                match (self, rhs) {
                    $((Self::$f(_, lhs), Self::$f(_, rhs)) => Some(*lhs = *rhs),)+
                    _ => None,
                }
            }
        }
    };
}

impl_delta!(
    FbDelta,
    FourBar,
    (P0x, f64, p0x, p0x_mut),
    (P0y, f64, p0y, p0y_mut),
    (A, f64, a, a_mut),
    (L1, f64, l1, l1_mut),
    (L2, f64, l2, l2_mut),
    (L3, f64, l3, l3_mut),
    (L4, f64, l4, l4_mut),
    (L5, f64, l5, l5_mut),
    (G, f64, g, g_mut),
    (Inv, bool, inv, inv_mut),
);

pub(crate) struct Undo<D: Delta> {
    undo: Vec<D>,
    redo: Vec<D>,
    last: Option<D::State>,
}

impl<D: Delta> Default for Undo<D> {
    fn default() -> Self {
        Self { undo: Vec::new(), redo: Vec::new(), last: None }
    }
}

impl<D: Delta> Undo<D> {
    pub(crate) fn able_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub(crate) fn able_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub(crate) fn fetch(&mut self, state: &D::State) {
        let Some(base) = &self.last else {
            self.last = Some(state.clone());
            return;
        };
        let Some(delta) = D::delta(base, state) else { return };
        if self
            .undo
            .last_mut()
            .and_then(|d| d.try_merge(&delta))
            .is_none()
        {
            self.undo.push(delta);
        }
        self.redo.clear();
        self.last = Some(state.clone());
    }

    pub(crate) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.last = None;
    }

    pub(crate) fn undo(&mut self, state: &mut D::State) {
        let Some(delta) = self.undo.pop() else { return };
        delta.undo(state);
        self.redo.push(delta);
        self.last = Some(state.clone());
    }

    pub(crate) fn redo(&mut self, state: &mut D::State) {
        let Some(delta) = self.redo.pop() else { return };
        delta.redo(state);
        self.undo.push(delta);
        self.last = Some(state.clone());
    }
}
