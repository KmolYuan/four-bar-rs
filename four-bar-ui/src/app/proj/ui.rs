use super::*;
use efd::na;

const JOINT_COLOR: Color32 = Color32::from_rgb(93, 69, 56);
const LINK_COLOR: Color32 = Color32::from_rgb(165, 151, 132);

fn pick_color(i: usize) -> Color32 {
    use plot2d::{Color as _, Palette as _};
    let (r, g, b) = plot2d::Palette99::pick(i).to_rgba().rgb();
    Color32::from_rgb(r, g, b).gamma_multiply(0.8)
}

fn draw_joint<F>(ui: &mut plot::PlotUi, p: [f64; 2], fixed: bool, point_f: F)
where
    F: Fn(plot::Points) -> plot::Points,
{
    use plot::MarkerShape::*;
    let p = plot::Points::new(p)
        .radius(if fixed { 10. } else { 5. })
        .shape(if fixed { Up } else { Circle })
        .color(JOINT_COLOR);
    ui.points(point_f(p));
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

fn draw_link3d(ui: &mut plot::PlotUi, oc: [f64; 3], points: &[[f64; 3]], is_main: bool) {
    let width = if is_main { 3. } else { 1. };
    let oc = na::Point3::from(oc);
    let iter = points.windows(2).flat_map(|w| {
        let a = na::Point3::from(w[0]) - oc;
        let b = na::Point3::from(w[1]) - oc;
        let axis = a.cross(&b).normalize();
        let angle = a.normalize().dot(&b.normalize()).acos();
        const N: usize = 150;
        let step = angle / N as f64;
        (0..=N).map(move |i| {
            let p = na::UnitQuaternion::from_scaled_axis(axis * i as f64 * step) * a;
            [oc.x + p.x, oc.y + p.y, oc.z + p.z]
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
    draw_sline(ui, oc.z, iter, |line| line.width(width).color(LINK_COLOR));
}

pub(crate) trait ProjPlot<D: efd::EfdDim> {
    fn delta_plot(
        &self,
        ui: &mut plot::PlotUi,
        joints: Option<&[efd::Coord<D>; 5]>,
        curves: &[[efd::Coord<D>; 3]],
        is_main: bool,
    );
}

impl ProjPlot<efd::D2> for FourBar {
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
            for (js, fixed) in [(&joints[2..], false), (&joints[..2], true)] {
                for &[x, y] in js {
                    draw_joint(ui, [x, y], fixed, |p| p);
                }
            }
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

impl ProjPlot<efd::D3> for SFourBar {
    fn delta_plot(
        &self,
        ui: &mut plot::PlotUi,
        joints: Option<&[efd::Coord<efd::D3>; 5]>,
        curves: &[[efd::Coord<efd::D3>; 3]],
        is_main: bool,
    ) {
        const N: usize = 150;
        const STEP: f64 = std::f64::consts::TAU / N as f64;
        let r = self.r();
        let oc @ [ox, oy, oz] = self.oc();
        draw_joint(ui, [ox, oy], true, |p| p.shape(plot::MarkerShape::Diamond));
        let circle = (0..=N)
            .map(|i| i as f64 * STEP)
            .map(|t| [r * t.cos() + ox, r * t.sin() + oy])
            .collect::<Vec<_>>();
        ui.line(plot::Line::new(circle).style(plot::LineStyle::dashed_dense()));
        if let Some(joints) = joints {
            draw_link3d(ui, oc, &[joints[0], joints[2]], is_main);
            draw_link3d(ui, oc, &[joints[1], joints[3]], is_main);
            draw_link3d(ui, oc, &joints[2..], is_main);
            for (js, fixed) in [(&joints[2..], false), (&joints[..2], true)] {
                for &[x, y, z] in js {
                    draw_joint(ui, [x, y], fixed, |p| p.filled(z > oz));
                }
            }
        }
        for (i, name) in ["Driver joint", "Follower joint", "Coupler joint"]
            .into_iter()
            .enumerate()
        {
            let color = pick_color(i);
            let iter = curves.iter().map(|c| c[i]);
            draw_sline(ui, oz, iter, |s| s.name(name).width(3.).color(color));
        }
    }
}

pub(crate) trait ProjUi {
    fn delta_ui(&mut self, ui: &mut Ui, cfg: &Cfg) -> Response;
}

// A dummy UI function for angles.
fn angle(ui: &mut Ui, label: &str, val: &mut f64, _int: f64) -> Response {
    super::angle(ui, label, val, "")
}

macro_rules! impl_ui {
    ($name:ty, $(($m_mut: ident, $ui:ident, $des:literal),)+
        .., $(($p_m_mut: ident, $p_ui:ident, $p_des:literal),)+
        .., $(($b_m_mut: ident, $b_des:literal)),+ $(,)?) => {
        impl ProjUi for $name {
            fn delta_ui(&mut self, ui: &mut Ui, cfg: &Cfg) -> Response {
                let mut res = $($ui(ui, $des, self.$m_mut(), cfg.int))|+;
                ui.heading("Parameters");
                res |= $($p_ui(ui, $p_des, self.$p_m_mut(), cfg.int))|+;
                res | $(ui.checkbox(self.$b_m_mut(), $b_des))|+
            }
        }
    };
}

impl_ui!(
    FourBar,
    (p0x_mut, unit, "X Offset: "),
    (p0y_mut, unit, "Y Offset: "),
    (a_mut, angle, "Rotation: "),
    ..,
    (l1_mut, nonzero_f, "Ground: "),
    (l2_mut, nonzero_f, "Driver: "),
    (l3_mut, nonzero_f, "Coupler: "),
    (l4_mut, nonzero_f, "Follower: "),
    (l5_mut, nonzero_f, "Extended: "),
    (g_mut, angle, "Extended angle: "),
    ..,
    (inv_mut, "Invert follower and coupler"),
);
impl_ui!(
    SFourBar,
    (ox_mut, unit, "X Offset: "),
    (oy_mut, unit, "Y Offset: "),
    (oz_mut, unit, "Z Offset: "),
    (r_mut, unit, "Radius: "),
    (p0i_mut, angle, "Polar angle: "),
    (p0j_mut, angle, "Azimuth angle: "),
    (a_mut, angle, "Rotation: "),
    ..,
    (l1_mut, angle, "Ground: "),
    (l2_mut, angle, "Driver: "),
    (l3_mut, angle, "Coupler: "),
    (l4_mut, angle, "Follower: "),
    (l5_mut, angle, "Extended: "),
    (g_mut, angle, "Extended angle: "),
    ..,
    (inv_mut, "Invert follower and coupler"),
);
