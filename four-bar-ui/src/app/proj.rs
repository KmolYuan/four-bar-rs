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

#[allow(dead_code)]
#[derive(Default)]
struct Cache {
    changed: bool,
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
            cache: Cache { changed: true, ..Cache::default() },
        }
    }
}

impl ProjInner {
    fn ui(&mut self, ui: &mut Ui, pivot: &mut Pivot, interval: f64, n: usize) {
        let fb = &mut self.four_bar;
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
                    io::save_csv_ask(&get_curve(pivot));
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
            let res = unit(ui, "X Offset: ", fb.p0x_mut(), interval)
                | unit(ui, "Y Offset: ", fb.p0y_mut(), interval)
                | angle(ui, "Rotation: ", fb.a_mut(), "");
            self.cache.changed |= res.changed();
        });
        ui.group(|ui| {
            ui.heading("Parameters");
            let res = link(ui, "Ground: ", fb.l0_mut(), interval)
                | link(ui, "Driver: ", fb.l1_mut(), interval)
                | link(ui, "Coupler: ", fb.l2_mut(), interval)
                | link(ui, "Follower: ", fb.l3_mut(), interval)
                | link(ui, "Extended: ", fb.l4_mut(), interval)
                | angle(ui, "Angle: ", fb.g_mut(), "")
                | ui.checkbox(fb.inv_mut(), "Invert follower and coupler");
            self.cache.changed |= res.changed();
        });
        ui.group(|ui| {
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
            if let Some([s, e]) = self.four_bar.angle_bound() {
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
            let res = angle(ui, "Theta: ", &mut self.angles.theta2, "")
                | angle(ui, "Omega: ", &mut self.angles.omega2, "/s")
                | angle(ui, "Alpha: ", &mut self.angles.alpha2, "/s²");
            self.cache.changed |= res.changed();
            if ui.button("⚽ Dynamics").clicked() {
                self.angles.open = !self.angles.open;
            }
        });
        if self.angles.omega2 != 0. {
            self.angles.theta2 += self.angles.omega2 / 60.;
            self.cache.changed = true;
            ui.ctx().request_repaint();
        }
    }

    fn cache(&mut self, n: usize) {
        // Recalculation
        self.cache.changed = false;
        let m = Mechanism::new(&self.four_bar);
        m.apply(self.angles.theta2, [0, 1, 2, 3, 4], &mut self.cache.joints);
        self.cache.curves = if let Some([start, _]) = self.four_bar.angle_bound() {
            m.curve_all(start, start + TAU, n)
        } else {
            Default::default()
        };
        let step = 360. / n as f64;
        self.cache.theta3 = self.cache.curves[0]
            .iter()
            .zip(&self.cache.curves[1])
            .enumerate()
            .map(|(i, ([x1, y1], [x2, y2]))| {
                plot::Value::new(i as f64 * step, (y1 - y2).atan2(x1 - x2).to_degrees())
            })
            .collect();
        self.cache.theta4 = self.cache.curves[1]
            .iter()
            .enumerate()
            .map(|(i, [x, y])| {
                let y = (y - self.cache.joints[1][1])
                    .atan2(x - self.cache.joints[1][0])
                    .to_degrees();
                plot::Value::new(i as f64 * step, y)
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
        const NAMES: &[&str] = &["Crank pivot", "Follower pivot", "Coupler pivot"];
        for (path, name) in self.cache.curves.iter().zip(NAMES) {
            let line = plot::Line::new(as_values(path))
                .name(format!("{}:{}", name, i))
                .width(3.);
            ui.line(line);
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
            io::save_ron(&proj.four_bar, path);
        } else {
            let name = proj.path.name();
            let four_bar = proj.four_bar.clone();
            drop(proj);
            let proj_cloned = self.clone();
            io::save_ron_ask(&four_bar, &name, move |path| proj_cloned.set_path(path));
        }
    }

    fn show(&self, ui: &mut Ui, pivot: &mut Pivot, interval: f64, n: usize) {
        self.show_proj(ui, pivot, interval, n);
        self.dynamics(ui);
    }

    fn show_proj(&self, ui: &mut Ui, pivot: &mut Pivot, interval: f64, n: usize) {
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
        ui.label("Linkage type:");
        ui.label(proj.four_bar.ty().name());
        ui.checkbox(&mut proj.hide, "Hide 👁");
        ui.add_enabled_ui(!proj.hide, |ui| proj.ui(ui, pivot, interval, n));
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
        Window::new("⚽ Dynamics")
            .open(&mut proj.angles.open)
            .show(ui.ctx(), |ui| {
                plot(ui, "plot_theta3", "Theta3", theta3);
                plot(ui, "plot_theta4", "Theta4", theta4);
            });
    }

    fn plot(&self, ui: &mut plot::PlotUi, i: usize, id: usize, n: usize) {
        self.0.write().unwrap().plot(ui, i, id, n);
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
                io::open_ron(move |path, s| {
                    if let Ok(fb) = ron::from_str(&s) {
                        queue.push(Some(path), fb);
                    }
                });
            }
            if ui.button("🗋 New").clicked() {
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
        if self.select(ui) {
            ui.group(|ui| self.list[self.current].show(ui, &mut self.pivot, interval, n));
        } else {
            ui.heading("No project here!");
            ui.label("Please open or create a project.");
        }
    }

    pub fn select(&mut self, ui: &mut Ui) -> bool {
        if !self.is_empty() {
            ui.horizontal_wrapped(|ui| {
                for (i, proj) in self.list.iter().enumerate() {
                    ui.selectable_value(&mut self.current, i, proj.name());
                }
            });
            true
        } else {
            false
        }
    }

    pub fn current_curve(&self, n: usize) -> Vec<[f64; 2]> {
        Mechanism::new(&self.list[self.current].0.read().unwrap().four_bar).curve(0., TAU, n)
    }

    pub fn plot(&mut self, ui: &mut plot::PlotUi, n: usize) {
        if !self.queue.0.read().unwrap().is_empty() {
            self.list.append(&mut *self.queue.0.write().unwrap());
            self.current = self.len() - 1;
        }
        for (i, proj) in self.list.iter().enumerate() {
            proj.plot(ui, i, self.current, n);
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
