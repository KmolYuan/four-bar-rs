use super::{impl_proj::Cache, *};
use four_bar::{
    efd::na,
    mech::{AngleBound, Stat},
};
use std::iter::zip;

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
        let angle = (a.dot(&b) / (a.norm() * b.norm())).acos();
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

fn state_curves_style(s: egui_plot::Line) -> egui_plot::Line {
    s.name(CURVE_NAME[2])
        .width(3.)
        .color(pick_color(2))
        .style(egui_plot::LineStyle::dashed_dense())
}

fn plot2d_basic(ui: &mut egui_plot::PlotUi, cache: &Cache<2>, is_main: bool) {
    // Plot mechanism
    if let Some(joints) = cache.joints {
        draw_link2d(ui, &[joints[0], joints[2]], is_main);
        draw_link2d(ui, &[joints[1], joints[3]], is_main);
        draw_link2d(ui, &joints[2..], is_main);
        for (js, fixed) in [(&joints[2..], false), (&joints[..2], true)] {
            for &[x, y] in js {
                draw_joint(ui, [x, y], fixed, |p| p);
            }
        }
    }
    // Plot curves
    for (i, name) in CURVE_NAME.iter().enumerate() {
        let iter = cache.curves.iter().map(|c| c[i]).collect::<Vec<_>>();
        let line = egui_plot::Line::new(iter)
            .name(name)
            .width(3.)
            .color(pick_color(i));
        ui.line(line);
    }
}

pub(crate) trait ProjPlot<const D: usize> {
    fn proj_plot(&self, ui: &mut egui_plot::PlotUi, cache: &Cache<D>, is_main: bool);
}

impl ProjPlot<2> for FourBar {
    fn proj_plot(&self, ui: &mut egui_plot::PlotUi, cache: &Cache<2>, is_main: bool) {
        plot2d_basic(ui, cache, is_main);
        for line in &cache.state_curves {
            ui.line(state_curves_style(egui_plot::Line::new(line.clone())));
        }
    }
}

impl ProjPlot<2> for MFourBar {
    fn proj_plot(&self, ui: &mut egui_plot::PlotUi, cache: &Cache<2>, is_main: bool) {
        plot2d_basic(ui, cache, is_main);
        let bound = ui.plot_bounds();
        let scale = bound.width().min(bound.height()) / 2.;
        let mut pose = Vec::with_capacity(cache.curves.len());
        for ([.., p], v) in zip(&cache.curves, &cache.state_curves[0]) {
            let q = std::array::from_fn(|i| p[i] + scale * v[i]);
            ui.line(state_curves_style(egui_plot::Line::new(vec![*p, q])));
            pose.push(q);
        }
        ui.line(state_curves_style(egui_plot::Line::new(pose)));
    }
}

impl ProjPlot<3> for SFourBar {
    fn proj_plot(&self, ui: &mut egui_plot::PlotUi, cache: &Cache<3>, is_main: bool) {
        const N: usize = 150;
        const STEP: f64 = std::f64::consts::TAU / N as f64;
        let r = self.unnorm.r;
        let sc @ [ox, oy, oz] = self.sc();
        // Plot sphere center
        draw_joint(ui, [ox, oy], true, |p| {
            p.shape(egui_plot::MarkerShape::Diamond)
        });
        // Plot sphere
        let circle = (0..=N)
            .map(|i| i as f64 * STEP)
            .map(|t| [r * t.cos() + ox, r * t.sin() + oy])
            .collect::<Vec<_>>();
        ui.line(egui_plot::Line::new(circle).style(egui_plot::LineStyle::dashed_dense()));
        // Plot mechanism
        if let Some(joints) = cache.joints {
            draw_link3d(ui, sc, &[joints[0], joints[2]], is_main);
            draw_link3d(ui, sc, &[joints[1], joints[3]], is_main);
            draw_link3d(ui, sc, &joints[2..], is_main);
            for (js, fixed) in [(&joints[2..], false), (&joints[..2], true)] {
                for &[x, y, z] in js {
                    draw_joint(ui, [x, y], fixed, |p| p.filled(z > oz));
                }
            }
        }
        // Plot state curves
        for line in &cache.state_curves {
            draw_sline(ui, oz, line.iter().copied(), state_curves_style);
        }
        // Plot curves
        for (i, name) in CURVE_NAME.iter().enumerate() {
            let color = pick_color(i);
            let iter = cache.curves.iter().map(|c| c[i]);
            draw_sline(ui, oz, iter, |s| s.name(name).width(3.).color(color));
        }
    }
}

pub(crate) trait ProjUi {
    fn proj_ui(&mut self, ui: &mut Ui) -> Response;
}

// A dummy UI function for angles.
fn angle(ui: &mut Ui, label: &str, val: &mut f64, _int: f64) -> Response {
    super::angle(ui, label, val, "")
}

fn stat_combo(ui: &mut Ui, stat: &mut Stat, bound: AngleBound) -> Response {
    let states = bound.get_states();
    if !states.contains(stat) {
        *stat = match stat {
            Stat::C1B1 => Stat::C1B1, // always valid
            Stat::C1B2 => Stat::C2B1, // maybe [C1B1, C2B1]
            Stat::C2B1 => Stat::C1B2, // maybe [C1B1, C1B2]
            Stat::C2B2 => Stat::C1B2, // maybe [C1B1, C2B1] or [C1B1, C1B2]
        };
    }
    ui.horizontal(|ui| {
        ui.label("State");
        states
            .into_iter()
            .map(|label| {
                ui.selectable_value(stat, label, label.name_uppercase())
                    .on_hover_text(format!("{label}"))
            })
            .reduce(|a, b| a | b)
            .unwrap()
    })
    .inner
}

macro_rules! impl_ui {
    ($name:ty, $($header:literal, $(($field:ident $(.$unnorm:ident)?, $ui:ident, $des:literal)),+,)+) => {
        impl ProjUi for $name {
            fn proj_ui(&mut self, ui: &mut Ui) -> Response {
                $(({
                    ui.heading($header);
                    $($ui(ui, $des, &mut self.$($unnorm.)?$field, 1.))|+
                }))|+ | ({
                    let bound = self.angle_bound();
                    stat_combo(ui, &mut self.stat, bound)
                })
            }
        }
    };
}

impl_ui!(
    FourBar,
    "Geometric Variables",
    (p1x.unnorm, unit, "X Offset: "),
    (p1y.unnorm, unit, "Y Offset: "),
    (a.unnorm, angle, "Rotation: "),
    "Parameters",
    (l1, nonzero_f, "Ground: "),
    (l2.unnorm, nonzero_f, "Driver: "),
    (l3, nonzero_f, "Coupler: "),
    (l4, nonzero_f, "Follower: "),
    (l5, nonzero_f, "Extended: "),
    (g, angle, "Extended angle: "),
);
impl_ui!(
    MFourBar,
    "Geometric Variables",
    (p1x.unnorm, unit, "X Offset: "),
    (p1y.unnorm, unit, "Y Offset: "),
    (a.unnorm, angle, "Rotation: "),
    "Parameters",
    (l1, nonzero_f, "Ground: "),
    (l2.unnorm, nonzero_f, "Driver: "),
    (l3, nonzero_f, "Coupler: "),
    (l4, nonzero_f, "Follower: "),
    (l5, nonzero_f, "Extended: "),
    (g, angle, "Extended angle: "),
    (e, angle, "Motion angle: "),
);
impl_ui!(
    SFourBar,
    "Geometric Variables",
    (ox.unnorm, unit, "X Offset: "),
    (oy.unnorm, unit, "Y Offset: "),
    (oz.unnorm, unit, "Z Offset: "),
    (r.unnorm, nonzero_f, "Radius: "),
    (p1i.unnorm, angle, "Polar angle: "),
    (p1j.unnorm, angle, "Azimuth angle: "),
    (a.unnorm, angle, "Rotation: "),
    "Parameters",
    (l1, angle, "Ground: "),
    (l2, angle, "Driver: "),
    (l3, angle, "Coupler: "),
    (l4, angle, "Follower: "),
    (l5, angle, "Extended: "),
    (g, angle, "Extended angle: "),
);
