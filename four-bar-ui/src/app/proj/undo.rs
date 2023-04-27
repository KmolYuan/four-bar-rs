macro_rules! impl_undo_redo {
    (@$(fn $method:ident($self:ident) { $ind:expr } => $($f:ident, $m:ident);+)+) => {$(
        fn $method(&$self, state: &mut Self::State) {
            *match $self.0 { $(Field::$f => state.$m()),+ } = $ind;
        }
    )+};
    ($(fn $method:ident($self:ident) { $ind:expr })+) => {
        impl_undo_redo!(@$(fn $method($self) { $ind } =>
            P0x, p0x_mut; P0y, p0y_mut; A, a_mut; L1, l1_mut; L2, l2_mut; L3, l3_mut; L4, l4_mut; L5, l5_mut; G, g_mut)+);
    };
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
    #[allow(unused_variables)]
    fn require_merge(&self, rhs: &Self) -> bool {
        false
    }
    #[allow(unused_variables)]
    fn merge(&mut self, rhs: Self) {}
}

#[derive(PartialEq)]
enum Field {
    P0x,
    P0y,
    A,
    L1,
    L2,
    L3,
    L4,
    L5,
    G,
}

pub(crate) struct FourBarDelta(Field, f64, f64);

impl Delta for FourBarDelta {
    type State = four_bar::FourBar;

    fn delta(a: &Self::State, b: &Self::State) -> Option<Self> {
        macro_rules! branch {
            ($($f:ident, $m:ident);+) => {
                match (a, b) {
                    $(_ if a.$m() != b.$m() => Some(Self(Field::$f, a.$m(), b.$m())),)+
                    _ => None,
                }
            };
        }
        branch!(P0x, p0x; P0y, p0y; A, a; L1, l1; L2, l2; L3, l3; L4, l4; L5, l5; G, g)
    }

    impl_undo_redo! {
        fn undo(self) { self.1 }
        fn redo(self) { self.2 }
    }

    fn require_merge(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }

    fn merge(&mut self, rhs: Self) {
        self.2 = rhs.2;
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
        if let Some(d) = self.undo.last_mut().filter(|d| d.require_merge(&delta)) {
            d.merge(delta);
        } else {
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
