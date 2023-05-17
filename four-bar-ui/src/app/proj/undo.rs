use super::*;
use eframe::egui::{Response, Ui};

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

pub(crate) trait DeltaUi {
    fn delta_ui(&mut self, ui: &mut Ui, cfg: &Cfg) -> Response;
}

// A dummy UI function for angles.
fn angle(ui: &mut Ui, label: &str, val: &mut f64, _int: f64) -> Response {
    super::angle(ui, label, val, "")
}

pub(crate) trait DeltaPlot<D: efd::EfdDim> {
    fn delta_plot(
        ui: &mut plot::PlotUi,
        joints: [efd::Coord<D>; 5],
        curves: &[[efd::Coord<D>; 3]],
        is_main: bool,
    );
}

macro_rules! impl_delta {
    ($ty_name:ident, $state:ident, $(($f:ident, $m:ident, $m_mut:ident, $ui:ident, $des:literal)),+,
        .., $(($b_f:ident, $b_m:ident, $b_m_mut:ident, $b_des:literal)),+ $(,)?) => {
        #[derive(PartialEq)]
        pub(crate) enum $ty_name {
            $($f(f64),)+
            $($b_f,)+
        }

        impl IntoDelta for $state {
            type Delta = $ty_name;
        }

        impl DeltaUi for $state {
            fn delta_ui(&mut self, ui: &mut Ui, cfg: &Cfg) -> Response {
                $($ui(ui, $des, self.$m_mut(), cfg.int))|+
                | $(ui.checkbox(self.$b_m_mut(), $b_des))|+
            }
        }

        impl Delta for $ty_name {
            type State = $state;

            fn delta(a: &Self::State, b: &Self::State) -> Option<Self> {
                match (a, b) {
                    $(_ if a.$m() != b.$m() => Some(Self::$f(b.$m() - a.$m())),)+
                    $(_ if a.$b_m() != b.$b_m() => Some(Self::$b_f),)+
                    _ => None,
                }
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
    (P0x, p0x, p0x_mut, unit, "X Offset: "),
    (P0y, p0y, p0y_mut, unit, "Y Offset: "),
    (A, a, a_mut, angle, "Rotation: "),
    (L1, l1, l1_mut, nonzero_f, "Ground: "),
    (L2, l2, l2_mut, nonzero_f, "Driver: "),
    (L3, l3, l3_mut, nonzero_f, "Coupler: "),
    (L4, l4, l4_mut, nonzero_f, "Follower: "),
    (L5, l5, l5_mut, nonzero_f, "Extended: "),
    (G, g, g_mut, angle, "Extended angle: "),
    ..,
    (Inv, inv, inv_mut, "Invert follower and coupler"),
);
impl_delta!(
    SFbDelta,
    SFourBar,
    (Ox, ox, ox_mut, unit, "X Offset: "),
    (Oy, oy, oy_mut, unit, "Y Offset: "),
    (Oz, oz, oz_mut, unit, "Z Offset: "),
    (R, r, r_mut, unit, "Radius: "),
    (P0i, p0i, p0i_mut, angle, "Polar angle: "),
    (P0j, p0j, p0j_mut, angle, "Azimuth angle: "),
    (A, a, a_mut, angle, "Rotation: "),
    (L1, l1, l1_mut, angle, "Ground: "),
    (L2, l2, l2_mut, angle, "Driver: "),
    (L3, l3, l3_mut, angle, "Coupler: "),
    (L4, l4, l4_mut, angle, "Follower: "),
    (L5, l5, l5_mut, angle, "Extended: "),
    (G, g, g_mut, angle, "Extended angle: "),
    ..,
    (Inv, inv, inv_mut, "Invert follower and coupler"),
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
        // FIXME: Use `is_some_and`
        if self.undo.last_mut().map(|d| d.try_merge(&delta)) != Some(true) {
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

impl DeltaPlot<efd::D2> for FourBar {
    fn delta_plot(
        ui: &mut plot::PlotUi,
        joints: [[f64; 2]; 5],
        curves: &[[[f64; 2]; 3]],
        is_main: bool,
    ) {
        draw_link(ui, &[joints[0], joints[2]], is_main);
        draw_link(ui, &[joints[1], joints[3]], is_main);
        draw_link(ui, &joints[2..], is_main);
        let float_j = plot::Points::new(joints[2..].to_vec())
            .radius(5.)
            .color(JOINT_COLOR);
        let fixed_j = plot::Points::new(joints[..2].to_vec())
            .radius(10.)
            .shape(plot::MarkerShape::Up)
            .color(JOINT_COLOR);
        ui.points(float_j);
        ui.points(fixed_j);
        for (i, name) in ["Driver joint", "Follower joint", "Coupler joint"]
            .into_iter()
            .enumerate()
        {
            let iter = curves.iter().map(|c| c[i]).collect::<Vec<_>>();
            ui.line(plot::Line::new(iter).name(name).width(3.));
        }
    }
}

impl DeltaPlot<efd::D3> for SFourBar {
    #[allow(unused_variables)]
    fn delta_plot(
        ui: &mut plot::PlotUi,
        joints: [efd::Coord<efd::D3>; 5],
        curves: &[[efd::Coord<efd::D3>; 3]],
        is_main: bool,
    ) {
        // TODO: 3D plot
    }
}
