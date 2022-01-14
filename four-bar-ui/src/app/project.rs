use super::{
    io_ctx::IoCtx,
    widgets::{angle, link, unit},
};
use crate::{as_values::as_values, dump_csv};
use eframe::egui::{
    plot::{Line, MarkerShape, PlotUi, Points, Polygon},
    Button, Color32, ComboBox, Ui,
};
use four_bar::{synthesis::open_curve, FourBar, Mechanism};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
    sync::{Arc, RwLock},
};

macro_rules! ext {
    () => {
        "ron"
    };
}

const JOINT_COLOR: Color32 = Color32::from_rgb(93, 69, 56);
const LINK_COLOR: Color32 = Color32::from_rgb(165, 151, 132);
const FMT: &str = "Rusty Object Notation";
const CSV_FMT: &str = "Delimiter-Separated Values";
const EXT: &[&str] = &[ext!()];
const CSV_EXT: &[&str] = &["csv", "txt"];

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

fn draw_link(ui: &mut PlotUi, points: &[[f64; 2]], is_main: bool) {
    let values = as_values(points);
    let width = if is_main { 3. } else { 1. };
    if points.len() == 2 {
        ui.line(Line::new(values).width(width).color(LINK_COLOR));
    } else {
        let polygon = Polygon::new(values)
            .width(width)
            .fill_alpha(if is_main { 0.8 } else { 0.2 })
            .color(LINK_COLOR);
        ui.polygon(polygon);
    }
}

#[derive(Deserialize, Serialize, PartialEq)]
pub(crate) enum Pivot {
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

impl From<Option<String>> for ProjName {
    fn from(path: Option<String>) -> Self {
        match path {
            Some(path) => Self::Path(path),
            None => Self::Untitled,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct ProjInner {
    hide: bool,
    path: ProjName,
    four_bar: FourBar,
}

impl Default for ProjInner {
    fn default() -> Self {
        Self {
            hide: false,
            path: ProjName::default(),
            four_bar: FourBar::example(),
        }
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub(crate) struct Project(Arc<RwLock<ProjInner>>);

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

    fn name(&self) -> String {
        let proj = self.0.read().unwrap();
        match &proj.path {
            ProjName::Path(path) => filename(path),
            ProjName::Named(name) => with_ext(name),
            ProjName::Untitled => concat!["untitled.", ext!()].to_string(),
        }
    }

    fn save(&self) {
        let s = ron::to_string(&self.0.read().unwrap().four_bar).unwrap();
        if let ProjName::Path(path) = &self.0.read().unwrap().path {
            IoCtx::save(&s, path);
        } else {
            let proj = self.clone();
            IoCtx::save_ask(&s, &self.name(), FMT, EXT, move |path| proj.set_path(path));
        }
    }

    fn four_bar_ui(&self, ui: &mut Ui, pivot: &mut Pivot, interval: f64, n: usize) {
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
        ui.checkbox(&mut proj.hide, "Hide 👁");
        ui.add_enabled_ui(!proj.hide, |ui| {
            let fb = &mut proj.four_bar;
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    if ui.button("💾 Save Curve").clicked() {
                        let m = Mechanism::four_bar(fb);
                        let curve = m.four_bar_loop_all(0., n);
                        let p = match pivot {
                            Pivot::Driver => &curve[0],
                            Pivot::Follower => &curve[1],
                            Pivot::Coupler => &curve[2],
                        };
                        let s = dump_csv(&open_curve(p)).unwrap();
                        IoCtx::save_ask(&s, "curve.csv", CSV_FMT, CSV_EXT, |_| ());
                    }
                    ComboBox::from_label("")
                        .selected_text(pivot.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(pivot, Pivot::Coupler, Pivot::Coupler.name());
                            ui.selectable_value(pivot, Pivot::Driver, Pivot::Driver.name());
                            ui.selectable_value(pivot, Pivot::Follower, Pivot::Follower.name());
                        });
                });
            });
            ui.group(|ui| {
                ui.heading("Offset");
                if ui
                    .add_enabled(!fb.is_aligned(), Button::new("Reset"))
                    .clicked()
                {
                    fb.align();
                }
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
        });
    }

    fn plot(&self, ui: &mut PlotUi, i: usize, id: usize, angle: f64, n: usize) {
        let m = Mechanism::four_bar(&self.0.read().unwrap().four_bar);
        let is_main = i == id;
        let mut joints = [[0.; 2]; 5];
        m.apply(angle, [0, 1, 2, 3, 4], &mut joints);
        draw_link(ui, &[joints[0], joints[2]], is_main);
        draw_link(ui, &[joints[1], joints[3]], is_main);
        draw_link(ui, &joints[2..], is_main);
        let float_j = Points::new(as_values(&joints[2..]))
            .radius(5.)
            .color(JOINT_COLOR);
        let fixed_j = Points::new(as_values(&joints[..2]))
            .radius(10.)
            .shape(MarkerShape::Up)
            .color(JOINT_COLOR);
        ui.points(float_j);
        ui.points(fixed_j);
        let curve = m.four_bar_loop_all(0., n);
        let path_names = ["Crank pivot", "Follower pivot", "Coupler pivot"];
        for (path, name) in curve.iter().zip(path_names) {
            let line = Line::new(as_values(path))
                .name(format!("{}:{}", name, i))
                .width(3.);
            ui.line(line);
        }
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub(crate) struct Queue(Arc<RwLock<Vec<Project>>>);

impl Queue {
    pub(crate) fn push(&self, path: Option<String>, four_bar: FourBar) {
        self.0.write().unwrap().push(Project::new(path, four_bar));
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Projects {
    list: Vec<Project>,
    queue: Queue,
    pivot: Pivot,
    current: usize,
}

impl Projects {
    pub(crate) fn push(&mut self, path: Option<String>, four_bar: FourBar) {
        self.list.push(Project::new(path, four_bar));
    }

    pub(crate) fn push_default(&mut self) {
        self.list.push(Project::default());
    }

    pub(crate) fn queue(&self) -> Queue {
        self.queue.clone()
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, interval: f64, n: usize) {
        #[cfg(not(target_arch = "wasm32"))]
        for file in ui.ctx().input().raw.dropped_files.iter() {
            if let Some(path) = &file.path {
                let s = std::fs::read_to_string(path).unwrap_or_default();
                if let Ok(fb) = ron::from_str(&s) {
                    let path = path.to_str().unwrap().to_string();
                    self.push(Some(path), fb);
                }
            }
        }
        ui.horizontal(|ui| {
            if ui.button("🖴 Open").clicked() {
                let queue = self.queue();
                IoCtx::open(FMT, EXT, move |path, s| {
                    if let Ok(fb) = ron::from_str(&s) {
                        queue.push(Some(path), fb);
                    }
                });
            }
            if ui.button("➕ New").clicked() {
                self.push_default();
                self.current = self.len() - 1;
            }
            if !self.is_empty() {
                if ui.button("💾 Save").clicked() {
                    self[self.current].save();
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
        if self.is_empty() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            for (i, proj) in self.list.iter().enumerate() {
                ui.selectable_value(&mut self.current, i, proj.name());
            }
        });
        ui.group(|ui| self.list[self.current].four_bar_ui(ui, &mut self.pivot, interval, n));
    }

    pub(crate) fn plot(&self, ui: &mut PlotUi, angle: f64, n: usize) {
        for (i, proj) in self.list.iter().enumerate() {
            proj.plot(ui, i, self.current, angle, n);
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
