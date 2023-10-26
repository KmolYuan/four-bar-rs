use super::*;

pub(crate) trait Delta: Sized {
    type State: Clone;

    fn delta(a: &Self::State, b: &Self::State) -> Option<Self>;
    fn undo(&self, state: &mut Self::State);
    fn redo(&self, state: &mut Self::State);
    // Return true if successfully merged
    fn try_merge(&mut self, rhs: &Self) -> bool;
}

pub(crate) trait IntoDelta: Clone {
    type Delta: Delta<State = Self>;
}

macro_rules! impl_delta {
    ($ty_name:ident, $state:ident, $(($f:ident, $(@$unnorm: ident,)? $m:ident),)+
        .., $(($b_f:ident, $b_m:ident)),+ $(,)?) => {
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
                    $(_ if a.$($unnorm.)?$m != b.$($unnorm.)?$m => Self::$f(b.$($unnorm.)?$m - a.$($unnorm.)?$m),)+
                    $(_ if a.$b_m != b.$b_m => Self::$b_f,)+
                    _ => None?,
                })
            }

            fn undo(&self, state: &mut Self::State) {
                match self {
                    $(Self::$f(v) => state.$($unnorm.)?$m -= *v,)+
                    $(Self::$b_f => state.$b_m = !state.$b_m,)+
                }
            }

            fn redo(&self, state: &mut Self::State) {
                match self {
                    $(Self::$f(v) => state.$($unnorm.)?$m += *v,)+
                    $(Self::$b_f => state.$b_m = !state.$b_m,)+
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
    (P1x, @unnorm, p1x),
    (P1y, @unnorm, p1y),
    (A, @unnorm, a),
    (L1, l1),
    (L2, @unnorm, l2),
    (L3, l3),
    (L4, l4),
    (L5, l5),
    (G, g),
    ..,
    (Stat, stat),
);
impl_delta!(
    SFbDelta,
    SFourBar,
    (Ox, @unnorm, ox),
    (Oy, @unnorm, oy),
    (Oz, @unnorm, oz),
    (R, @unnorm, r),
    (P1i, @unnorm, p1i),
    (P1j, @unnorm, p1j),
    (A, @unnorm, a),
    (L1, l1),
    (L2, l2),
    (L3, l3),
    (L4, l4),
    (L5, l5),
    (G, g),
    ..,
    (Stat, stat),
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
        let Some(delta) = D::delta(base, state) else {
            return;
        };
        if !self.undo.last_mut().is_some_and(|d| d.try_merge(&delta)) {
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
