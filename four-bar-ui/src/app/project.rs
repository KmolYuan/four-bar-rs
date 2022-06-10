use super::{
    io_ctx::IoCtx,
    widgets::{angle, link, unit},
};
use crate::{as_values::as_values, dump_csv, ext};
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
fn open(file: impl AsRef<Path>) -> Option<FourBar> {
    if let Ok(s) = std::fs::read_to_string(file) {
        ron::from_str(&s).ok()
    } else {
        None
    }
}

#[cfg(target_arch = "wasm32")]
fn open(_file: impl AsRef<Path>) -> Option<FourBar> {
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

#[derive(Deserialize, Serialize, PartialEq)]
pub enum Pivot {
    Driver,
    Follower,
    Coupler,
}

impl Default for Pivot {
    fn default() -> Self {
        Self::Coupler
    }
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

#[derive(Deserialize, Serialize, Clone)]
enum ProjName {
    Path(String),
    Named(String),
    Untitled,
}

impl Default for ProjName {
    fn default() -> Self {
        Self::Untitled
    }
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
    open: bool,
}

#[allow(dead_code)] // TODO
#[derive(Default)]
struct Cache {
    cached: bool,
    joints: [[f64; 2]; 5],
    curves: [Vec<[f64; 2]>; 3],
    theta3: Vec<plot::Value>,
    theta4: Vec<plot::Value>,
    omega3: Vec<plot::Value>,
    omega4: Vec<plot::Value>,
    alpha3: Vec<plot::Value>,
    alpha4: Vec<plot::Value>,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct ProjInner {
    path: ProjName,
    four_bar: FourBar,
    angles: Angles,
    hide: bool,
    #[serde(skip)]
    cache: Cache,
}

impl Default for ProjInner {
    fn default() -> Self {
        Self {
            path: Default::default(),
            four_bar: FourBar::example(),
            angles: Default::default(),
            hide: false,
            cache: Default::default(),
        }
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct Project(Arc<RwLock<ProjInner>>);

impl Project {
    fn new(path: Option<String>, four_bar: FourBar) -> Self {
        let inner = ProjInner {
            path: ProjName::from(path),
            four_bar,
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

    fn reload(&self) {
        let mut proj = self.0.write().unwrap();
        if let ProjName::Path(path) = &proj.path {
            if let Some(four_bar) = open(path) {
                proj.four_bar = four_bar;
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
            IoCtx::save_ron(&proj.four_bar, path);
        } else {
            let name = proj.path.name();
            let four_bar = proj.four_bar.clone();
            drop(proj);
            let proj_cloned = self.clone();
            IoCtx::save_ron_ask(&four_bar, &name, move |path| proj_cloned.set_path(path));
        }
    }

    fn show(&self, ui: &mut Ui, pivot: &mut Pivot, interval: f64, n: usize) {
        let mut proj = self.0.write().unwrap();
        ui.horizontal(|ui| match &mut proj.path {
            ProjName::Path(path) => {
                let filename = filename(path);
                if ui.small_button("✏").on_hover_text("Rename path").clicked() {
                    proj.path = ProjName::Named(filename);
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
                    proj.path = ProjName::Named(name);
                }
            }
        });
        ui.label(format!("Linkage type: {}", proj.four_bar.class()));
        ui.checkbox(&mut proj.hide, "Hide 👁");
        ui.add_enabled_ui(!proj.hide, |ui| {
            let angles_o = proj.angles.clone();
            let fb_o = proj.four_bar.clone();
            let fb = &mut proj.four_bar;
            let get_curve = |pivot: &Pivot| {
                let m = Mechanism::new(fb);
                let [curve1, curve2, curve3] = m.curve_all(0., TAU, n);
                curve::get_valid_part(&match pivot {
                    Pivot::Driver => curve1,
                    Pivot::Follower => curve2,
                    Pivot::Coupler => curve3,
                })
            };
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("💾 Save Curve").clicked() {
                        IoCtx::save_csv_ask(&get_curve(pivot));
                    }
                    ComboBox::from_label("")
                        .selected_text(pivot.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(pivot, Pivot::Coupler, Pivot::Coupler.name());
                            ui.selectable_value(pivot, Pivot::Driver, Pivot::Driver.name());
                            ui.selectable_value(pivot, Pivot::Follower, Pivot::Follower.name());
                        });
                });
                if ui.button("🗐 Copy Curve to CSV").clicked() {
                    ui.output().copied_text = dump_csv(&get_curve(pivot)).unwrap();
                }
                let curve = get_curve(pivot);
                if !curve.is_empty() {
                    ui.label(format!("Cusps: {}", curve::cusp(&curve, false)));
                    ui.label(format!("Crunodes: {}", curve::crunode(&curve)));
                }
            });
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Offset");
                    let btn = ui.add_enabled(!fb.is_aligned(), Button::new("Reset"));
                    if btn.clicked() {
                        fb.align();
                    }
                });
                if ui.button("Normalize").clicked() {
                    fb.normalize();
                }
                ui.add(unit("X Offset: ", &mut fb.p0[0], interval));
                ui.add(unit("Y Offset: ", &mut fb.p0[1], interval));
                angle(ui, "Rotation: ", &mut fb.a, "");
            });
            ui.group(|ui| {
                ui.heading("Parameters");
                ui.add(link("Ground: ", &mut fb.l0, interval));
                ui.add(link("Driver: ", &mut fb.l1, interval));
                ui.add(link("Coupler: ", &mut fb.l2, interval));
                ui.add(link("Follower: ", &mut fb.l3, interval));
                ui.checkbox(&mut fb.inv, "Invert follower and coupler");
            });
            ui.group(|ui| {
                ui.heading("Coupler");
                ui.add(link("Extended: ", &mut fb.l4, interval));
                angle(ui, "Angle: ", &mut fb.g, "");
            });
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Angle");
                    reset_button(ui, &mut proj.angles);
                });
                angle(ui, "Theta: ", &mut proj.angles.theta2, "");
                angle(ui, "Omega: ", &mut proj.angles.omega2, "/s");
                angle(ui, "Alpha: ", &mut proj.angles.alpha2, "/s²");
                if ui.button("🌋 Dynamics").clicked() {
                    proj.angles.open = !proj.angles.open;
                }
            });
            if !proj.cache.cached || fb_o != proj.four_bar || angles_o != proj.angles {
                // Recalculation
                proj.cache.cached = true;
                let m = Mechanism::new(&proj.four_bar);
                m.apply(proj.angles.theta2, [0, 1, 2, 3, 4], &mut proj.cache.joints);
                proj.cache.curves = m.curve_all(0., TAU, n);
                let step = 360. / n as f64;
                proj.cache.theta3 = proj.cache.curves[0]
                    .iter()
                    .zip(&proj.cache.curves[1])
                    .enumerate()
                    .map(|(i, ([x1, y1], [x2, y2]))| {
                        plot::Value::new(i as f64 * step, (y1 - y2).atan2(x1 - x2).to_degrees())
                    })
                    .collect();
                proj.cache.theta4 = proj.cache.curves[1]
                    .iter()
                    .enumerate()
                    .map(|(i, [x, y])| {
                        let y = (y - proj.cache.joints[1][1])
                            .atan2(x - proj.cache.joints[1][0])
                            .to_degrees();
                        plot::Value::new(i as f64 * step, y)
                    })
                    .collect();
            }
        });
        drop(proj);
        self.dynamics(ui);
    }

    fn dynamics(&self, ui: &mut Ui) {
        fn plot(ui: &mut Ui, id: &str, title: &str, values: plot::Values) {
            let line = plot::Line::new(values).color(Color32::BLUE);
            ui.vertical(|ui| {
                ui.heading(title);
                plot::Plot::new(id)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_scroll(false)
                    .height(200.)
                    .show(ui, |ui| ui.line(line));
            });
        }
        let mut proj = self.0.write().unwrap();
        let theta3 = plot::Values::from_values(proj.cache.theta3.clone());
        let theta4 = plot::Values::from_values(proj.cache.theta4.clone());
        Window::new("🌋 Dynamics")
            .open(&mut proj.angles.open)
            .show(ui.ctx(), |ui| {
                plot(ui, "plot_theta3", "Theta3", theta3);
                plot(ui, "plot_theta4", "Theta4", theta4);
            });
        if proj.angles.omega2 != 0. {
            proj.angles.theta2 += proj.angles.omega2 / 60.;
            ui.ctx().request_repaint();
        }
    }

    fn plot(&self, ui: &mut plot::PlotUi, i: usize, id: usize) {
        let proj = self.0.read().unwrap();
        if proj.hide {
            return;
        }
        let is_main = i == id;
        draw_link(ui, &[proj.cache.joints[0], proj.cache.joints[2]], is_main);
        draw_link(ui, &[proj.cache.joints[1], proj.cache.joints[3]], is_main);
        draw_link(ui, &proj.cache.joints[2..], is_main);
        let float_j = plot::Points::new(as_values(&proj.cache.joints[2..]))
            .radius(5.)
            .color(JOINT_COLOR);
        let fixed_j = plot::Points::new(as_values(&proj.cache.joints[..2]))
            .radius(10.)
            .shape(plot::MarkerShape::Up)
            .color(JOINT_COLOR);
        ui.points(float_j);
        ui.points(fixed_j);
        const NAMES: &[&str] = &["Crank pivot", "Follower pivot", "Coupler pivot"];
        for (path, name) in proj.cache.curves.iter().zip(NAMES) {
            let line = plot::Line::new(as_values(path))
                .name(format!("{}:{}", name, i))
                .width(3.);
            ui.line(line);
        }
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct Queue(Arc<RwLock<Vec<Project>>>);

impl Queue {
    pub fn push(&self, path: Option<String>, four_bar: FourBar) {
        self.0.write().unwrap().push(Project::new(path, four_bar));
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Projects {
    list: Vec<Project>,
    queue: Queue,
    pivot: Pivot,
    current: usize,
}

impl Projects {
    pub fn push(&mut self, path: Option<String>, four_bar: FourBar) {
        // Prevent opening duplicate project
        if match &path {
            None => true,
            Some(path) => !self.list.iter().any(|p| match p.path() {
                Some(path_old) => path_old == *path,
                None => false,
            }),
        } {
            self.list.push(Project::new(path, four_bar));
            self.current = self.len() - 1;
        }
    }

    pub fn push_default(&mut self) {
        self.list.push(Project::default());
        self.current = self.len() - 1;
    }

    pub fn open(&mut self, file: impl AsRef<Path>) {
        let path = file.as_ref().to_str().unwrap().to_string();
        if let Some(four_bar) = open(file) {
            self.push(Some(path), four_bar);
            self.current = self.len() - 1;
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
                IoCtx::open_ron(move |path, s| {
                    if let Ok(fb) = ron::from_str(&s) {
                        queue.push(Some(path), fb);
                    }
                });
            }
            if ui.button("➕ New").clicked() {
                self.push_default();
            }
            if !self.is_empty() {
                if ui.button("💾 Save").clicked() {
                    self.list[self.current].save();
                }
                if ui.button("✖ Close").clicked() {
                    self.list.remove(self.current);
                    if self.current > 0 {
                        self.current -= 1;
                    }
                }
            }
        });
        if !self.queue.0.read().unwrap().is_empty() {
            self.list.append(&mut *self.queue.0.write().unwrap());
            self.current = self.len() - 1;
        }
        if self.select(ui) {
            ui.group(|ui| self.list[self.current].show(ui, &mut self.pivot, interval, n));
        } else {
            let head = RichText::new("No project here!")
                .color(Color32::BLUE)
                .heading();
            ui.label(head);
            ui.colored_label(Color32::BLUE, "Please open or create a project.");
        }
    }

    pub fn select(&mut self, ui: &mut Ui) -> bool {
        match self.is_empty() {
            true => false,
            false => {
                ui.horizontal_wrapped(|ui| {
                    for (i, proj) in self.list.iter().enumerate() {
                        ui.selectable_value(&mut self.current, i, proj.name());
                    }
                    true
                })
                .inner
            }
        }
    }

    pub fn current_curve(&self, n: usize) -> Vec<[f64; 2]> {
        Mechanism::new(&self.list[self.current].0.read().unwrap().four_bar).curve(0., TAU, n)
    }

    pub fn plot(&self, ui: &mut plot::PlotUi) {
        for (i, proj) in self.list.iter().enumerate() {
            proj.plot(ui, i, self.current);
        }
    }

    pub fn reload(&self) {
        for p in self.list.iter() {
            p.reload();
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
