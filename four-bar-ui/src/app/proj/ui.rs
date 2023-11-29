use super::*;
use four_bar::{
    efd,
    efd::na,
    fb::{PlanarLoop as _, Stat},
};

const JOINT_COLOR: Color32 = Color32::from_rgb(93, 69, 56);
const LINK_COLOR: Color32 = Color32::from_rgb(165, 151, 132);
const CURVE_NAME: &[&str] = &["Driver Curve", "Follower Curve", "Coupler Curve"];

fn pick_color(i: usize) -> Color32 {
    use four_bar::plot::{Color as _, Palette as _, Palette99};
    let (r, g, b) = Palette99::pick(i).to_rgba().rgb();
    Color32::from_rgb(r, g, b).gamma_multiply(0.8)
}

fn draw_joint<F>(ui: &mut egui_plot::PlotUi, p: [f64; 2], fixed: bool, point_f: F)
where
    F: Fn(egui_plot::Points) -> egui_plot::Points,
{
    use egui_plot::MarkerShape::*;
    let p = egui_plot::Points::new(p)
        .radius(if fixed { 10. } else { 5. })
        .shape(if fixed { Up } else { Circle })
        .color(JOINT_COLOR);
    ui.points(point_f(p));
}

fn draw_link2d(ui: &mut egui_plot::PlotUi, line: &[[f64; 2]], is_main: bool) {
    let width = if is_main { 3. } else { 1. };
    if line.len() == 2 {
        let line = egui_plot::Line::new(line.to_vec())
            .width(width)
            .color(LINK_COLOR);
        ui.line(line);
    } else {
        let polygon = egui_plot::Polygon::new(line.to_vec())
            .stroke((width, LINK_COLOR))
            .fill_color(LINK_COLOR.gamma_multiply(if is_main { 0.8 } else { 0.2 }));
        ui.polygon(polygon);
    }
}

fn draw_sline<I, F>(ui: &mut egui_plot::PlotUi, oz: f64, line: I, line_f: F)
where
    I: IntoIterator<Item = [f64; 3]>,
    F: Fn(egui_plot::Line) -> egui_plot::Line,
{
    let mut line = line.into_iter();
    loop {
        let is_front = std::cell::OnceCell::new();
        let line = line
            .by_ref()
            .take_while(|&[.., z]| {
                let stat = z >= oz;
                *is_front.get_or_init(|| stat) == stat
            })
            .map(|[x, y, _]| [x, y])
            .collect::<Vec<_>>();
        if line.is_empty() {
            break;
        }
        let mut line = line_f(egui_plot::Line::new(line));
        if !is_front.get().unwrap() {
            line = line.style(egui_plot::LineStyle::dashed_dense());
        }
        ui.line(line);
    }
}

fn draw_link3d(ui: &mut egui_plot::PlotUi, sc: [f64; 3], points: &[[f64; 3]], is_main: bool) {
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
        let polygon = egui_plot::Polygon::new(points)
            .stroke((width, LINK_COLOR))
            .fill_color(LINK_COLOR.gamma_multiply(if is_main { 0.8 } else { 0.2 }));
        ui.polygon(polygon);
    }
    draw_sline(ui, sc.z, iter, |line| line.width(width).color(LINK_COLOR));
}

pub(crate) trait ProjPlot<D: efd::EfdDim> {
    fn proj_plot(
        &self,
        ui: &mut egui_plot::PlotUi,
        joints: Option<&[efd::Coord<D>; 5]>,
        curves: &[[efd::Coord<D>; 3]],
        is_main: bool,
    );
}

impl ProjPlot<efd::D2> for FourBar {
    fn proj_plot(
        &self,
        ui: &mut egui_plot::PlotUi,
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
        for (i, name) in CURVE_NAME.iter().enumerate() {
            let iter = curves.iter().map(|c| c[i]).collect::<Vec<_>>();
            let line = egui_plot::Line::new(iter)
                .name(name)
                .width(3.)
                .color(pick_color(i));
            ui.line(line);
        }
    }
}

impl ProjPlot<efd::D3> for SFourBar {
    fn proj_plot(
        &self,
        ui: &mut egui_plot::PlotUi,
        joints: Option<&[efd::Coord<efd::D3>; 5]>,
        curves: &[[efd::Coord<efd::D3>; 3]],
        is_main: bool,
    ) {
        const N: usize = 150;
        const STEP: f64 = std::f64::consts::TAU / N as f64;
        let r = self.unnorm.r;
        let sc @ [ox, oy, oz] = self.sc();
        draw_joint(ui, [ox, oy], true, |p| {
            p.shape(egui_plot::MarkerShape::Diamond)
        });
        let circle = (0..=N)
            .map(|i| i as f64 * STEP)
            .map(|t| [r * t.cos() + ox, r * t.sin() + oy])
            .collect::<Vec<_>>();
        ui.line(egui_plot::Line::new(circle).style(egui_plot::LineStyle::dashed_dense()));
        if let Some(joints) = joints {
            draw_link3d(ui, sc, &[joints[0], joints[2]], is_main);
            draw_link3d(ui, sc, &[joints[1], joints[3]], is_main);
            draw_link3d(ui, sc, &joints[2..], is_main);
            for (js, fixed) in [(&joints[2..], false), (&joints[..2], true)] {
                for &[x, y, z] in js {
                    draw_joint(ui, [x, y], fixed, |p| p.filled(z > oz));
                }
            }
        }
        for (i, name) in CURVE_NAME.iter().enumerate() {
            let color = pick_color(i);
            let iter = curves.iter().map(|c| c[i]);
            draw_sline(ui, oz, iter, |s| s.name(name).width(3.).color(color));
        }
    }
}

pub(crate) trait ProjUi {
    fn proj_ui(&mut self, ui: &mut Ui, cfg: &Cfg) -> Response;
}

// A dummy UI function for angles.
fn angle(ui: &mut Ui, label: &str, val: &mut f64, _int: f64) -> Response {
    super::angle(ui, label, val, "")
}

fn stat_combo(res: &mut Response, ui: &mut Ui, stat: &mut Stat, has_branch: bool) {
    let states = if has_branch {
        Stat::list4()
    } else {
        match stat {
            Stat::C1B2 => *stat = Stat::C1B1,
            Stat::C2B2 => *stat = Stat::C2B1,
            _ => (),
        }
        Stat::list2()
    };
    ui.horizontal(|ui| {
        ui.label("State");
        for label in states {
            *res |= ui
                .selectable_value(stat, label, label.name_uppercase())
                .on_hover_text(format!("{label}"));
        }
    });
}

macro_rules! impl_ui {
    ($name:ty, $(($m_mut: ident, $ui:ident, $des:literal),)+
        .., $(($(@$unnorm: ident,)? $p_m_mut: ident, $p_ui:ident, $p_des:literal),)+
        .., $($stat: ident),+ $(,)?) => {
        impl ProjUi for $name {
            fn proj_ui(&mut self, ui: &mut Ui, cfg: &Cfg) -> Response {
                let mut res = $($ui(ui, $des, &mut self.unnorm.$m_mut, cfg.int))|+;
                ui.heading("Parameters");
                res |= $($p_ui(ui, $p_des, &mut self.$($unnorm.)?$p_m_mut, cfg.int))|+;
                let has_branch = self.has_branch();
                $(stat_combo(&mut res, ui, &mut self.$stat, has_branch);)+
                res
            }
        }
    };
}

impl_ui!(
    FourBar,
    (p1x, unit, "X Offset: "),
    (p1y, unit, "Y Offset: "),
    (a, angle, "Rotation: "),
    ..,
    (l1, nonzero_f, "Ground: "),
    (@unnorm, l2, nonzero_f, "Driver: "),
    (l3, nonzero_f, "Coupler: "),
    (l4, nonzero_f, "Follower: "),
    (l5, nonzero_f, "Extended: "),
    (g, angle, "Extended angle: "),
    ..,
    stat,
);
impl_ui!(
    SFourBar,
    (ox, unit, "X Offset: "),
    (oy, unit, "Y Offset: "),
    (oz, unit, "Z Offset: "),
    (r, nonzero_f, "Radius: "),
    (p1i, angle, "Polar angle: "),
    (p1j, angle, "Azimuth angle: "),
    (a, angle, "Rotation: "),
    ..,
    (l1, angle, "Ground: "),
    (l2, angle, "Driver: "),
    (l3, angle, "Coupler: "),
    (l4, angle, "Follower: "),
    (l5, angle, "Extended: "),
    (g, angle, "Extended angle: "),
    ..,
    stat,
);
