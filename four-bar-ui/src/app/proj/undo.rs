use super::*;

pub(crate) trait Delta: Sized {
    type State: Clone;

    fn delta(a: &Self::State, b: &Self::State) -> Option<Self>;
    fn undo(&self, state: &mut Self::State);
    fn redo(&self, state: &mut Self::State);
    fn try_merge(&mut self, rhs: &Self) -> bool;
}

pub(crate) trait IntoDelta: Clone {
    type Delta: Delta<State = Self>;
}

macro_rules! impl_delta {
    ($ty_name:ident, $state:ident, $(($f:ident, $m:ident, $m_mut:ident),)+
        .., $(($b_f:ident, $b_m:ident, $b_m_mut:ident)),+ $(,)?) => {
        #[derive(PartialEq)]
        pub(crate) enum $ty_name {
            $($f(f64),)+
            $($b_f,)+
        }

        impl IntoDelta for $state {
            type Delta = $ty_name;
        }

        impl Delta for $ty_name {
            type State = $state;

            fn delta(a: &Self::State, b: &Self::State) -> Option<Self> {
                Some(match (a, b) {
                    $(_ if a.$m() != b.$m() => Self::$f(b.$m() - a.$m()),)+
                    $(_ if a.$b_m() != b.$b_m() => Self::$b_f,)+
                    _ => None?,
                })
            }

            fn undo(&self, state: &mut Self::State) {
                match self {
                    $(Self::$f(v) => *state.$m_mut() -= *v,)+
                    $(Self::$b_f => *state.$b_m_mut() = !state.$b_m(),)+
                }
            }

            fn redo(&self, state: &mut Self::State) {
                match self {
                    $(Self::$f(v) => *state.$m_mut() += *v,)+
                    $(Self::$b_f => *state.$b_m_mut() = !state.$b_m(),)+
                }
            }

            fn try_merge(&mut self, rhs: &Self) -> bool {
                match (self, rhs) {
                    $((Self::$f(lhs), Self::$f(rhs)) => {*lhs += *rhs; true},)+
                    _ => false,
                }
            }
        }
    };
}

impl_delta!(
    FbDelta,
    FourBar,
    (P0x, p0x, p0x_mut),
    (P0y, p0y, p0y_mut),
    (A, a, a_mut),
    (L1, l1, l1_mut),
    (L2, l2, l2_mut),
    (L3, l3, l3_mut),
    (L4, l4, l4_mut),
    (L5, l5, l5_mut),
    (G, g, g_mut),
    ..,
    (Inv, inv, inv_mut),
);
impl_delta!(
    SFbDelta,
    SFourBar,
    (Ox, ox, ox_mut),
    (Oy, oy, oy_mut),
    (Oz, oz, oz_mut),
    (R, r, r_mut),
    (P0i, p0i, p0i_mut),
    (P0j, p0j, p0j_mut),
    (A, a, a_mut),
    (L1, l1, l1_mut),
    (L2, l2, l2_mut),
    (L3, l3, l3_mut),
    (L4, l4, l4_mut),
    (L5, l5, l5_mut),
    (G, g, g_mut),
    ..,
    (Inv, inv, inv_mut),
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
            self.last.replace(state.clone());
            return;
        };
        let Some(delta) = D::delta(base, state) else { return };
        if self.undo.last_mut().is_some_and(|d| d.try_merge(&delta)) {
            self.undo.push(delta);
        }
        self.redo.clear();
        self.last.replace(state.clone());
    }

    pub(crate) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.last.take();
    }

    pub(crate) fn undo(&mut self, state: &mut D::State) {
        let Some(delta) = self.undo.pop() else { return };
        delta.undo(state);
        self.redo.push(delta);
        self.last.replace(state.clone());
    }

    pub(crate) fn redo(&mut self, state: &mut D::State) {
        let Some(delta) = self.redo.pop() else { return };
        delta.redo(state);
        self.undo.push(delta);
        self.last.replace(state.clone());
    }
}
