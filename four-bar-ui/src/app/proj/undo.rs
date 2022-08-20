use four_bar::FourBar;

pub trait Delta: Sized {
    type State: Clone;

    fn delta(a: &Self::State, b: &Self::State) -> Option<Self>;
    fn change(&self, target: &mut Self::State);
}

#[derive(Debug)]
enum Field {
    P0x,
    P0y,
    A,
    L0,
    L1,
    L2,
    L3,
    L4,
    G,
}

#[derive(Debug)]
pub struct FourBarDelta(Field, f64);

impl Delta for FourBarDelta {
    type State = FourBar;

    fn delta(a: &Self::State, b: &Self::State) -> Option<Self> {
        macro_rules! branch {
            ($a:ident, $b:ident => $($f:ident, $m:ident);+ $(;)?) => {
                match ($a, $b) {
                    $((a, b) if a.$m() != b.$m() => Some(Self(Field::$f, a.$m())),)+
                    _ => None,
                }
            };
        }
        branch! { a, b => P0x, p0x; P0y, p0y; A, a; L0, l0; L1, l1; L2, l2; L3, l3; L4, l4; G, g }
    }

    fn change(&self, state: &mut Self::State) {
        *match self.0 {
            Field::P0x => state.p0x_mut(),
            Field::P0y => state.p0y_mut(),
            Field::A => state.a_mut(),
            Field::L0 => state.l0_mut(),
            Field::L1 => state.l1_mut(),
            Field::L2 => state.l2_mut(),
            Field::L3 => state.l3_mut(),
            Field::L4 => state.l4_mut(),
            Field::G => state.g_mut(),
        } = self.1;
    }
}

#[derive(Debug)]
pub struct Undo<D: Delta> {
    undo: Vec<D>,
    redo: Vec<D>,
    base: Option<D::State>,
    time: f64,
}

impl<S: Delta + std::fmt::Debug> Default for Undo<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: Delta + std::fmt::Debug> Undo<D> {
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            base: None,
            time: 0.,
        }
    }

    pub fn is_able_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn is_able_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn fetch(&mut self, time: f64, state: &D::State) {
        if let Some(base) = &self.base {
            if let Some(delta) = D::delta(base, state) {
                if time - self.time < 5. && !self.undo.is_empty() {
                    return;
                }
                self.undo.push(delta);
                self.redo.clear();
                self.base = Some(state.clone());
                self.time = time;
            }
        } else {
            self.base = Some(state.clone());
            self.time = time;
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn undo(&mut self, state: &mut D::State) {
        let base = self.base.as_mut().unwrap();
        *base = state.clone();
        self.undo.pop().unwrap().change(state);
        self.redo.push(D::delta(base, state).unwrap());
        *base = state.clone();
    }

    pub fn redo(&mut self, state: &mut D::State) {
        let base = self.base.as_mut().unwrap();
        *base = state.clone();
        self.redo.pop().unwrap().change(state);
        self.undo.push(D::delta(base, state).unwrap());
        *base = state.clone();
    }
}
