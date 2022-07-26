use super::{
    as_values::as_values,
    csv::dump_csv,
    io,
    widgets::{angle, link, unit},
};
use crate::ext;
use eframe::egui::*;
use four_bar::{curve, FourBar, Mechanism};
use serde::{Deserialize, Serialize};
use std::{
    f64::consts::TAU,
    ops::{Deref, DerefMut},
    path::Path,
    sync::{Arc, RwLock},
};

const JOINT_COLOR: Color32 = Color32::from_rgb(93, 69, 56);
const LINK_COLOR: Color32 = Color32::from_rgb(165, 151, 132);

#[cfg(not(target_arch = "wasm32"))]
fn pre_open(file: impl AsRef<Path>) -> Option<FourBar> {
    if let Ok(s) = std::fs::read_to_string(file) {
        ron::from_str(&s).ok()
    } else {
        None
    }
}

#[cfg(target_arch = "wasm32")]
#[inline]
fn pre_open<F>(_file: F) -> Option<FourBar> {
    None
}

fn with_ext(name: &str) -> String {
    if name.ends_with(concat![".", ext!()]) {
        name.to_string()
    } else {
        name.to_string() + concat![".", ext!()]
    }
}

fn filename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string()
}

fn draw_link(ui: &mut plot::PlotUi, points: &[[f64; 2]], is_main: bool) {
    let values = as_values(points);
    let width = if is_main { 3. } else { 1. };
    if points.len() == 2 {
        ui.line(plot::Line::new(values).width(width).color(LINK_COLOR));
    } else {
        let polygon = plot::Polygon::new(values)
            .width(width)
            .fill_alpha(if is_main { 0.8 } else { 0.2 })
            .color(LINK_COLOR);
        ui.polygon(polygon);
    }
}

fn plot_values(ui: &mut plot::PlotUi, values: &[(f64, [f64; 3])], symbol: &str, use_rad: bool) {
    for i in 0..=2 {
        let values = if use_rad {
            let iter = values.iter().map(|(x, y)| plot::Value::new(*x, y[i]));
            plot::Values::from_values_iter(iter)
        } else {
            let iter = values
                .iter()
                .map(|(x, y)| plot::Value::new(x.to_degrees(), y[i].to_degrees()));
            plot::Values::from_values_iter(iter)
        };
        ui.line(plot::Line::new(values).name(format!("{}{}", symbol, i + 2)));
    }
}

#[derive(Default, Deserialize, Serialize, PartialEq)]
pub enum Pivot {
    Driver,
    Follower,
    #[default]
    Coupler,
}

impl Pivot {
    const fn name(&self) -> &str {
        match self {
            Pivot::Driver => "Driver",
            Pivot::Follower => "Follower",
            Pivot::Coupler => "Coupler",
        }
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
enum ProjName {
    Path(String),
    Named(String),
    #[default]
    Untitled,
}

impl ProjName {
    fn name(&self) -> String {
        match self {
            Self::Path(path) => filename(path),
            Self::Named(name) => with_ext(name),
            Self::Untitled => concat!["untitled.", ext!()].to_string(),
        }
    }
}

impl From<Option<String>> for ProjName {
    fn from(path: Option<String>) -> Self {
        match path {
            Some(path) => Self::Path(path),
            None => Self::Untitled,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Default)]
#[serde(default)]
struct Angles {
    theta2: f64,
    omega2: f64,
    alpha2: f64,
}

#[allow(dead_code)]
#[derive(Default)]
struct Cache {
    changed: bool,
    joints: [[f64; 2]; 5],
    curves: Vec<[[f64; 2]; 3]>,
    dynamics: Vec<(f64, [[f64; 3]; 3])>,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct ProjInner {
    path: ProjName,
    fb: FourBar,
    angles: Angles,
    hide: bool,
    angle_open: bool,
    angle_use_rad: bool,
    #[serde(skip)]
    cache: Cache,
}

impl Default for ProjInner {
    fn default() -> Self {
        Self {
            path: Default::default(),
            fb: FourBar::example(),
            angles: Default::default(),
            hide: false,
            angle_open: false,
            angle_use_rad: false,
            cache: Cache { changed: true, ..Cache::default() },
        }
    }
}

impl ProjInner {
    fn show(&mut self, ui: &mut Ui, pivot: &mut Pivot, interval: f64, n: usize) {
        ui.horizontal(|ui| match &mut self.path {
            ProjName::Path(path) => {
                let filename = filename(path);
                if ui.small_button("✏").on_hover_text("Rename path").clicked() {
                    self.path = ProjName::Named(filename);
                } else {
                    ui.label(&filename);
                }
            }
            ProjName::Named(name) => {
                ui.colored_label(Color32::RED, "Unsaved path");
                ui.text_edit_singleline(name);
            }
            ProjName::Untitled => {
                ui.colored_label(Color32::RED, "Unsaved path");
                let mut name = "untitled".to_string();
                if ui.text_edit_singleline(&mut name).changed() {
                    self.path = ProjName::Named(name);
                }
            }
        });
        ui.label("Linkage type:");
        ui.label(self.fb.ty().name());
        ui.checkbox(&mut self.hide, "Hide 👁");
        ui.add_enabled_ui(!self.hide, |ui| self.ui(ui, pivot, interval, n));
        Window::new("⚽ Dynamics")
            .open(&mut self.angle_open)
            .vscroll(true)
            .show(ui.ctx(), |ui| {
                let res = angle(ui, "Omega: ", &mut self.angles.omega2, "/s")
                    | angle(ui, "Alpha: ", &mut self.angles.alpha2, "/s²");
                self.cache.changed |= res.changed();
                ui.checkbox(&mut self.angle_use_rad, "Plot radians");
                for (i, (id, symbol, title)) in [
                    ("plot_theta", "theta", "Angle"),
                    ("plot_omega", "omega", "Angular Velocity"),
                    ("plot_alpha", "alpha", "Angular Acceleration"),
                ]
                .into_iter()
                .enumerate()
                {
                    let values = self
                        .cache
                        .dynamics
                        .iter()
                        .map(|(x, t)| (*x, t[i]))
                        .collect::<Vec<_>>();
                    ui.heading(title);
                    plot::Plot::new(id)
                        .legend(Default::default())
                        .height(200.)
                        .show(ui, |ui| {
                            plot_values(ui, &values, symbol, self.angle_use_rad)
                        });
                }
            });
    }

    fn ui(&mut self, ui: &mut Ui, pivot: &mut Pivot, interval: f64, n: usize) {
        let fb = &mut self.fb;
        let get_curve = |pivot: &Pivot| {
            let m = Mechanism::new(fb);
            let curve = m.curve_all(0., TAU, n);
            curve::get_valid_part(&match pivot {
                Pivot::Driver => curve.into_iter().map(|[c, _, _]| c).collect::<Vec<_>>(),
                Pivot::Follower => curve.into_iter().map(|[_, c, _]| c).collect::<Vec<_>>(),
                Pivot::Coupler => curve.into_iter().map(|[_, _, c]| c).collect::<Vec<_>>(),
            })
        };
        ui.heading("Curve");
        ui.horizontal(|ui| {
            ComboBox::from_label("")
                .selected_text(pivot.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(pivot, Pivot::Coupler, Pivot::Coupler.name());
                    ui.selectable_value(pivot, Pivot::Driver, Pivot::Driver.name());
                    ui.selectable_value(pivot, Pivot::Follower, Pivot::Follower.name());
                });
            if ui.small_button("💾").on_hover_text("Save").clicked() {
                io::save_csv_ask(&get_curve(pivot));
            }
            if ui.button("🗐").on_hover_text("Copy").clicked() {
                ui.output().copied_text = dump_csv(&get_curve(pivot)).unwrap();
            }
        });
        let curve = get_curve(pivot);
        if !curve.is_empty() {
            ui.label(format!("Crunodes: {}", curve::crunode(&curve)));
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("Offset");
            if ui
                .add_enabled(!fb.is_aligned(), Button::new("Reset"))
                .clicked()
            {
                fb.align();
                self.cache.changed = true;
            }
        });
        if ui.button("Normalize").clicked() {
            fb.normalize();
            self.cache.changed = true;
        }
        let mut res = unit(ui, "X Offset: ", fb.p0x_mut(), interval)
            | unit(ui, "Y Offset: ", fb.p0y_mut(), interval)
            | angle(ui, "Rotation: ", fb.a_mut(), "");
        ui.separator();
        ui.heading("Parameters");
        res |= link(ui, "Ground: ", fb.l0_mut(), interval)
            | link(ui, "Driver: ", fb.l1_mut(), interval)
            | link(ui, "Coupler: ", fb.l2_mut(), interval)
            | link(ui, "Follower: ", fb.l3_mut(), interval)
            | link(ui, "Extended: ", fb.l4_mut(), interval)
            | angle(ui, "Angle: ", fb.g_mut(), "")
            | ui.checkbox(fb.inv_mut(), "Invert follower and coupler");
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("Angle");
            if ui
                .add_enabled(self.angles != Default::default(), Button::new("Reset"))
                .clicked()
            {
                self.angles = Default::default();
                self.cache.changed = true;
            }
        });
        if let Some([s, e]) = self.fb.angle_bound() {
            ui.group(|ui| {
                ui.label("Click to copy angle bounds:");
                let mut copy_btn = |s: f64, e: f64, suffix: &str| {
                    ui.horizontal(|ui| {
                        let s_str = format!("{:.04}", s);
                        if ui.selectable_label(false, &s_str).clicked() {
                            ui.output().copied_text = s_str;
                        }
                        let e_str = format!("{:.04}", e);
                        if ui.selectable_label(false, &e_str).clicked() {
                            ui.output().copied_text = e_str;
                        }
                        ui.label(suffix);
                    });
                };
                copy_btn(s, e, "rad");
                copy_btn(s.to_degrees(), e.to_degrees(), "deg");
            });
        }
        res |= angle(ui, "Theta: ", &mut self.angles.theta2, "");
        self.cache.changed |= res.changed();
        if ui.button("⚽ Dynamics").clicked() {
            self.angle_open = !self.angle_open;
        }
    }

    fn cache(&mut self, n: usize) {
        // Recalculation
        self.cache.changed = false;
        let m = Mechanism::new(&self.fb);
        m.apply(self.angles.theta2, [0, 1, 2, 3, 4], &mut self.cache.joints);
        self.cache.curves = if let Some([start, _]) = self.fb.angle_bound() {
            m.curve_all(start, start + TAU, n)
        } else {
            Default::default()
        };
        let step = TAU / n as f64;
        self.cache.dynamics = self
            .cache
            .curves
            .iter()
            .enumerate()
            .map(|(i, [[x2, y2], [x3, y3], _])| {
                let [x0, y0] = self.cache.joints[0];
                let [x1, y1] = self.cache.joints[1];
                let theta2 = (y2 - y0).atan2(x2 - x0);
                let theta3 = (y3 - y2).atan2(x3 - x2);
                let theta4 = (y3 - y1).atan2(x3 - x1);
                let theta = [theta2, theta3, theta4];
                let omega2 = self.angles.omega2;
                let omega3 = -omega2 * self.fb.l1() * (theta4 - theta2).sin()
                    / (self.fb.l2() * (theta4 - theta3).sin() + f64::EPSILON);
                let omega4 = omega2 * self.fb.l1() * (theta3 - theta2).sin()
                    / (self.fb.l3() * (theta3 - theta4).sin() + f64::EPSILON);
                let omega = [omega2, omega3, omega4];
                let alpha2 = self.angles.alpha2;
                let alpha3 = (-self.fb.l1() * alpha2 * (theta4 - theta2).sin()
                    + self.fb.l1() * omega2 * omega2 * (theta4 - theta2).cos()
                    + self.fb.l2() * omega3 * omega3 * (theta4 - theta3).cos()
                    - self.fb.l3() * omega4 * omega4)
                    / self.fb.l2()
                    * (theta4 - theta3).sin();
                let alpha4 = (self.fb.l1() * alpha2 * (theta3 - theta2).sin()
                    - self.fb.l1() * omega2 * omega2 * (theta3 - theta2).cos()
                    + self.fb.l3() * omega4 * omega4 * (theta3 - theta4).cos()
                    - self.fb.l2() * omega3 * omega3)
                    / self.fb.l3()
                    * (theta3 - theta4).sin();
                let alpha = [alpha2, alpha3, alpha4];
                (i as f64 * step, [theta, omega, alpha])
            })
            .collect();
    }

    fn plot(&mut self, ui: &mut plot::PlotUi, i: usize, id: usize, n: usize) {
        if self.hide {
            return;
        }
        if self.cache.changed {
            self.cache(n);
        }
        let is_main = i == id;
        draw_link(ui, &[self.cache.joints[0], self.cache.joints[2]], is_main);
        draw_link(ui, &[self.cache.joints[1], self.cache.joints[3]], is_main);
        draw_link(ui, &self.cache.joints[2..], is_main);
        let float_j = plot::Points::new(as_values(&self.cache.joints[2..]))
            .radius(5.)
            .color(JOINT_COLOR);
        let fixed_j = plot::Points::new(as_values(&self.cache.joints[..2]))
            .radius(10.)
            .shape(plot::MarkerShape::Up)
            .color(JOINT_COLOR);
        ui.points(float_j);
        ui.points(fixed_j);
        for (i, name) in ["Crank pivot", "Follower pivot", "Coupler pivot"]
            .into_iter()
            .enumerate()
        {
            let iter = self
                .cache
                .curves
                .iter()
                .map(|c| c[i])
                .map(|[x, y]| plot::Value::new(x, y));
            let line = plot::Line::new(plot::Values::from_values_iter(iter))
                .name(format!("{}:{}", name, i))
                .width(3.);
            ui.line(line);
        }
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct Project(Arc<RwLock<ProjInner>>);

impl Project {
    fn new(path: Option<String>, fb: FourBar) -> Self {
        let inner = ProjInner {
            path: ProjName::from(path),
            fb,
            ..Default::default()
        };
        Self(Arc::new(RwLock::new(inner)))
    }

    fn set_path(&self, path: String) {
        self.0.write().unwrap().path = ProjName::Path(path);
    }

    pub fn path(&self) -> Option<String> {
        match &self.0.read().unwrap().path {
            ProjName::Path(path) => Some(path.clone()),
            _ => None,
        }
    }

    pub fn re_open(&self) {
        let mut proj = self.0.write().unwrap();
        if let ProjName::Path(path) = &proj.path {
            if let Some(fb) = pre_open(path) {
                proj.fb = fb;
            } else {
                proj.path = ProjName::Named(path.clone());
            }
        }
    }

    pub fn name(&self) -> String {
        self.0.read().unwrap().path.name()
    }

    fn save(&self) {
        let proj = self.0.read().unwrap();
        if let ProjName::Path(path) = &proj.path {
            io::save_ron(&proj.fb, path);
        } else {
            let name = proj.path.name();
            let fb = proj.fb.clone();
            drop(proj);
            let proj_cloned = self.clone();
            io::save_ron_ask(&fb, &name, move |path| proj_cloned.set_path(path));
        }
    }

    fn show(&self, ui: &mut Ui, pivot: &mut Pivot, interval: f64, n: usize) {
        self.0.write().unwrap().show(ui, pivot, interval, n);
    }

    fn plot(&self, ui: &mut plot::PlotUi, i: usize, id: usize, n: usize) {
        self.0.write().unwrap().plot(ui, i, id, n);
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct Queue(Arc<RwLock<Vec<Project>>>);

impl Queue {
    pub fn push(&self, path: Option<String>, fb: FourBar) {
        self.0.write().unwrap().push(Project::new(path, fb));
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Projects {
    list: Vec<Project>,
    queue: Queue,
    pivot: Pivot,
    curr: usize,
}

impl Projects {
    pub fn push(&mut self, path: Option<String>, fb: FourBar) {
        // Prevent opening duplicate project
        if match &path {
            None => true,
            Some(path) => !self.list.iter().any(|p| match p.path() {
                Some(path_old) => path_old == *path,
                None => false,
            }),
        } {
            self.list.push(Project::new(path, fb));
            self.curr = self.len() - 1;
        }
    }

    pub fn push_default(&mut self) {
        self.list.push(Project::default());
        self.curr = self.len() - 1;
    }

    pub fn open(&mut self, file: impl AsRef<Path>) {
        let path = file.as_ref().to_str().unwrap().to_string();
        if let Some(fb) = pre_open(file) {
            self.push(Some(path), fb);
            self.curr = self.len() - 1;
        }
    }

    pub fn queue(&self) -> Queue {
        self.queue.clone()
    }

    pub fn show(&mut self, ui: &mut Ui, interval: f64, n: usize) {
        #[cfg(not(target_arch = "wasm32"))]
        for file in ui.ctx().input().raw.dropped_files.iter() {
            if let Some(path) = &file.path {
                self.open(path);
            }
        }
        ui.horizontal(|ui| {
            if ui.button("🖴 Open").clicked() {
                let queue = self.queue();
                io::open_ron(move |path, s| {
                    if let Ok(fb) = ron::from_str(&s) {
                        queue.push(Some(path), fb);
                    }
                });
            }
            if ui.button("🗋 New").clicked() {
                self.push_default();
            }
        });
        if self.select(ui, true) {
            self.list[self.curr].show(ui, &mut self.pivot, interval, n);
        } else {
            ui.heading("No project here!");
            ui.label("Please open or create a project.");
        }
    }

    pub fn select(&mut self, ui: &mut Ui, show_btn: bool) -> bool {
        if !self.is_empty() {
            ui.horizontal(|ui| {
                ComboBox::from_label("")
                    .show_index(ui, &mut self.curr, self.list.len(), |i| self.list[i].name());
                if show_btn {
                    if ui.button("💾 Save").clicked() {
                        self.list[self.curr].save();
                    }
                    if ui.button("✖ Close").clicked() {
                        self.list.remove(self.curr);
                        if self.curr > 0 {
                            self.curr -= 1;
                        }
                    }
                }
            });
        }
        !self.is_empty()
    }

    pub fn current_curve(&self, n: usize) -> Vec<[f64; 2]> {
        let proj = self.list[self.curr].0.read().unwrap();
        curve::get_valid_part(&Mechanism::new(&proj.fb).curve(0., TAU, n))
    }

    pub fn plot(&mut self, ui: &mut plot::PlotUi, n: usize) {
        if !self.queue.0.read().unwrap().is_empty() {
            self.list.append(&mut *self.queue.0.write().unwrap());
            self.curr = self.len() - 1;
        }
        for (i, proj) in self.list.iter().enumerate() {
            proj.plot(ui, i, self.curr, n);
        }
    }
}

impl Deref for Projects {
    type Target = Vec<Project>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for Projects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}
