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
    ($ty_name:ident, $state:ident, $($m:ident $(.$unnorm:ident)?),+) => {
        #[derive(PartialEq)]
        #[allow(non_camel_case_types)]
        pub(crate) enum $ty_name {
            $($m(f64),)+
            stat(i8),
        }

        impl IntoDelta for $state {
            type Delta = $ty_name;
        }

        impl Delta for $ty_name {
            type State = $state;

            fn delta(a: &Self::State, b: &Self::State) -> Option<Self> {
                Some(match (a, b) {
                    $(_ if a.$($unnorm.)?$m != b.$($unnorm.)?$m => Self::$m(b.$($unnorm.)?$m - a.$($unnorm.)?$m),)+
                    _ if a.stat != b.stat => Self::stat(b.stat as i8 - a.stat as i8),
                    _ => None?,
                })
            }

            fn undo(&self, state: &mut Self::State) {
                match self {
                    $(Self::$m(v) => state.$($unnorm.)?$m -= *v,)+
                    Self::stat(v) => state.stat = ((state.stat as i8 - *v) as u8).try_into().unwrap(),
                }
            }

            fn redo(&self, state: &mut Self::State) {
                match self {
                    $(Self::$m(v) => state.$($unnorm.)?$m += *v,)+
                    Self::stat(v) => state.stat = ((state.stat as i8 + *v) as u8).try_into().unwrap(),
                }
            }

            fn try_merge(&mut self, rhs: &Self) -> bool {
                match (self, rhs) {
                    $((Self::$m(lhs), Self::$m(rhs)) => {*lhs += *rhs; true},)+
                    _ => false,
                }
            }
        }
    };
}

impl_delta!(FbDelta, FourBar, p1x.unnorm, p1y.unnorm, a.unnorm, l1, l2.unnorm, l3, l4, l5, g);
impl_delta!(MFbDelta, MFourBar, p1x.unnorm, p1y.unnorm, a.unnorm, l1, l2.unnorm, l3, l4, l5, g, e);
impl_delta!(
    SFbDelta, SFourBar, ox.unnorm, oy.unnorm, oz.unnorm, r.unnorm, p1i.unnorm, p1j.unnorm,
    a.unnorm, l1, l2, l3, l4, l5, g
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
        let Some(delta) = D::delta(base, state) else {
            return;
        };
        if !self.undo.last_mut().is_some_and(|d| d.try_merge(&delta)) {
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
