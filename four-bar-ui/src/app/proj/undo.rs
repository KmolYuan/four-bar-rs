use super::*;
use efd::na;
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

const JOINT_COLOR: Color32 = Color32::from_rgb(93, 69, 56);
const LINK_COLOR: Color32 = Color32::from_rgb(165, 151, 132);

fn pick_color(i: usize) -> Color32 {
    use plot2d::{Color as _, Palette as _};
    let (r, g, b) = plot2d::Palette99::pick(i).to_rgba().rgb();
    Color32::from_rgb(r, g, b).gamma_multiply(0.8)
}

fn draw_link2d(ui: &mut plot::PlotUi, points: &[[f64; 2]], is_main: bool) {
    let width = if is_main { 3. } else { 1. };
    if points.len() == 2 {
        let line = plot::Line::new(points.to_vec())
            .width(width)
            .color(LINK_COLOR);
        ui.line(line);
    } else {
        let polygon = plot::Polygon::new(points.to_vec())
            .width(width)
            .fill_alpha(if is_main { 0.8 } else { 0.2 })
            .color(LINK_COLOR);
        ui.polygon(polygon);
    }
}

fn draw_sline<I, F>(ui: &mut plot::PlotUi, oz: f64, iter: I, line_f: F)
where
    I: IntoIterator<Item = [f64; 3]>,
    F: Fn(plot::Line) -> plot::Line,
{
    let mut iter = iter.into_iter().peekable();
    let Some([.., first_z]) = iter.peek() else { return };
    let mut is_front = *first_z > oz;
    loop {
        let curve = iter
            .by_ref()
            .take_while(|[.., z]| if is_front { *z > oz } else { *z <= oz })
            .map(|[x, y, _]| [x, y])
            .collect::<Vec<_>>();
        if curve.is_empty() {
            break;
        }
        let mut line = line_f(plot::Line::new(curve));
        if !is_front {
            line = line.style(plot::LineStyle::dashed_dense());
        }
        ui.line(line);
        is_front = !is_front;
    }
}

fn draw_link3d(ui: &mut plot::PlotUi, sc: [f64; 3], points: &[[f64; 3]], is_main: bool) {
    let width = if is_main { 3. } else { 1. };
    let sc = na::Point3::from(sc);
    let iter = points.windows(2).flat_map(|w| {
        let a = na::Point3::from(w[0]) - sc;
        let b = na::Point3::from(w[1]) - sc;
        let axis = a.cross(&b).normalize();
        let angle = a.normalize().dot(&b.normalize()).acos();
        const N: usize = 150;
        let step = angle / N as f64;
        (0..=N).map(move |i| {
            let p = na::UnitQuaternion::from_scaled_axis(axis * i as f64 * step) * a;
            [sc.x + p.x, sc.y + p.y, sc.z + p.z]
        })
    });
    if points.len() > 2 {
        let points = iter.clone().map(|[x, y, _]| [x, y]).collect::<Vec<_>>();
        let polygon = plot::Polygon::new(points)
            .width(width)
            .fill_alpha(if is_main { 0.8 } else { 0.2 })
            .color(LINK_COLOR);
        ui.polygon(polygon);
    }
    draw_sline(ui, sc.z, iter, |line| line.width(width).color(LINK_COLOR));
}

pub(crate) trait DeltaPlot<D: efd::EfdDim> {
    fn delta_plot(
        &self,
        ui: &mut plot::PlotUi,
        joints: Option<&[efd::Coord<D>; 5]>,
        curves: &[[efd::Coord<D>; 3]],
        is_main: bool,
    );
}

impl DeltaPlot<efd::D2> for FourBar {
    fn delta_plot(
        &self,
        ui: &mut plot::PlotUi,
        joints: Option<&[[f64; 2]; 5]>,
        curves: &[[[f64; 2]; 3]],
        is_main: bool,
    ) {
        if let Some(joints) = joints {
            draw_link2d(ui, &[joints[0], joints[2]], is_main);
            draw_link2d(ui, &[joints[1], joints[3]], is_main);
            draw_link2d(ui, &joints[2..], is_main);
            let float_j = plot::Points::new(joints[2..].to_vec())
                .radius(5.)
                .color(JOINT_COLOR);
            let fixed_j = plot::Points::new(joints[..2].to_vec())
                .radius(10.)
                .shape(plot::MarkerShape::Up)
                .color(JOINT_COLOR);
            ui.points(float_j);
            ui.points(fixed_j);
        }
        for (i, name) in ["Driver joint", "Follower joint", "Coupler joint"]
            .into_iter()
            .enumerate()
        {
            let iter = curves.iter().map(|c| c[i]).collect::<Vec<_>>();
            let line = plot::Line::new(iter)
                .name(name)
                .width(3.)
                .color(pick_color(i));
            ui.line(line);
        }
    }
}

impl DeltaPlot<efd::D3> for SFourBar {
    #[allow(unused_variables)]
    fn delta_plot(
        &self,
        ui: &mut plot::PlotUi,
        joints: Option<&[efd::Coord<efd::D3>; 5]>,
        curves: &[[efd::Coord<efd::D3>; 3]],
        is_main: bool,
    ) {
        let proj = |curve: &[[f64; 3]]| {
            curve
                .iter()
                .filter_map(|&[x, y, z]| (z > self.oz()).then_some([x, y]))
                .collect::<Vec<_>>()
        };
        const N: usize = 150;
        const STEP: f64 = std::f64::consts::TAU / N as f64;
        let circle = (0..=N)
            .map(|i| {
                let t = i as f64 * STEP;
                [
                    self.r() * t.cos() + self.ox(),
                    self.r() * t.sin() + self.oy(),
                ]
            })
            .collect::<Vec<_>>();
        ui.line(plot::Line::new(circle).style(plot::LineStyle::dashed_dense()));
        if let Some(joints) = joints {
            let sc = self.oc();
            draw_link3d(ui, sc, &[joints[0], joints[2]], is_main);
            draw_link3d(ui, sc, &[joints[1], joints[3]], is_main);
            draw_link3d(ui, sc, &joints[2..], is_main);
            let float_j = plot::Points::new(proj(&joints[2..]))
                .radius(5.)
                .color(JOINT_COLOR);
            let fixed_j = plot::Points::new(proj(&joints[..2]))
                .radius(10.)
                .shape(plot::MarkerShape::Up)
                .color(JOINT_COLOR);
            ui.points(float_j);
            ui.points(fixed_j);
        }
        for (i, name) in ["Driver joint", "Follower joint", "Coupler joint"]
            .into_iter()
            .enumerate()
        {
            let color = pick_color(i);
            let iter = curves.iter().map(|c| c[i]);
            draw_sline(ui, self.oz(), iter, |s| s.name(name).width(3.).color(color));
        }
    }
}
