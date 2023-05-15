use four_bar::FourBar;

macro_rules! impl_undo_redo {
    ($(fn $method:ident, $inv:ident)+) => {$(
        pub(crate) fn $method(&mut self, state: &mut D::State) {
            let Some(delta) = self.$method.pop() else { return };
            delta.$method(state);
            self.$inv.push(delta);
            self.last = Some(state.clone());
        }
    )+};
}

pub(crate) trait Delta: Sized {
    type State: Clone;

    fn delta(a: &Self::State, b: &Self::State) -> Option<Self>;
    fn undo(&self, state: &mut Self::State);
    fn redo(&self, state: &mut Self::State);
    fn try_merge(&mut self, rhs: &Self) -> Option<()>;
}

#[derive(PartialEq)]
pub(crate) enum FourBarDelta {
    P0x(f64, f64),
    P0y(f64, f64),
    A(f64, f64),
    L1(f64, f64),
    L2(f64, f64),
    L3(f64, f64),
    L4(f64, f64),
    L5(f64, f64),
    G(f64, f64),
    Inv(bool, bool),
}

impl Delta for FourBarDelta {
    type State = FourBar;

    fn delta(a: &Self::State, b: &Self::State) -> Option<Self> {
        macro_rules! branch {
            ($($f:ident, $m:ident),+) => {
                match (a, b) {
                    $(_ if a.$m() != b.$m() => Some(Self::$f(a.$m(), b.$m())),)+
                    _ => None,
                }
            };
        }
        branch!(P0x, p0x, P0y, p0y, A, a, L1, l1, L2, l2, L3, l3, L4, l4, L5, l5, G, g, Inv, inv)
    }

    fn undo(&self, state: &mut Self::State) {
        macro_rules! branch {
            ($($f:ident, $m:ident),+) => {
                match self { $(Self::$f(v, _) => *state.$m() = *v,)+ }
            };
        }
        branch!(
            P0x, p0x_mut, P0y, p0y_mut, A, a_mut, L1, l1_mut, L2, l2_mut, L3, l3_mut, L4, l4_mut,
            L5, l5_mut, G, g_mut, Inv, inv_mut
        );
    }

    fn redo(&self, state: &mut Self::State) {
        macro_rules! branch {
            ($($f:ident, $m:ident),+) => {
                match self { $(Self::$f(_, v) => *state.$m() = *v,)+ }
            };
        }
        branch!(
            P0x, p0x_mut, P0y, p0y_mut, A, a_mut, L1, l1_mut, L2, l2_mut, L3, l3_mut, L4, l4_mut,
            L5, l5_mut, G, g_mut, Inv, inv_mut
        );
    }

    fn try_merge(&mut self, rhs: &Self) -> Option<()> {
        macro_rules! branch {
            ($($f:ident),+) => {
                match (self, rhs) {
                    $((Self::$f(_, lhs), Self::$f(_, rhs)) => Some(*lhs = *rhs),)+
                    _ => None,
                }
            };
        }
        branch!(P0x, P0y, A, L1, L2, L3, L4, L5, G, Inv)
    }
}

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

    impl_undo_redo! {
        fn undo, redo
        fn redo, undo
    }
}
