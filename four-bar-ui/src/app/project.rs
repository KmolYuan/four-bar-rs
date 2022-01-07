use super::io_ctx::IoCtx;
use crate::{
    app::widgets::{angle, link, unit},
    as_values::as_values,
    dump_csv,
};
use eframe::egui::{
    plot::{Line, PlotUi, Points, Polygon},
    Button, Color32, Ui,
};
use four_bar::{FourBar, Mechanism};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
    sync::{Arc, Mutex},
};

const JOINT_COLOR: Color32 = Color32::from_rgb(93, 69, 56);
const LINK_COLOR: Color32 = Color32::from_rgb(165, 151, 132);

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

fn draw_joints(ui: &mut PlotUi, points: &[[f64; 2]]) {
    ui.points(Points::new(as_values(points)).radius(5.).color(JOINT_COLOR));
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

#[derive(Default, Deserialize, Serialize)]
struct ProjectInner {
    lazy: bool,
    path: Option<String>,
    four_bar: FourBar,
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub(crate) struct Project(Arc<Mutex<ProjectInner>>);

impl Project {
    fn new(path: Option<String>, four_bar: FourBar) -> Self {
        let inner = ProjectInner {
            path,
            four_bar,
            ..Default::default()
        };
        Self(Arc::new(Mutex::new(inner)))
    }

    fn lazy() -> Self {
        let inner = ProjectInner {
            lazy: true,
            ..Default::default()
        };
        Self(Arc::new(Mutex::new(inner)))
    }

    pub(crate) fn set_proj(&self, path: Option<String>, four_bar: FourBar) {
        let mut proj = self.0.lock().unwrap();
        proj.lazy = false;
        proj.path = path;
        proj.four_bar = four_bar;
    }

    pub(crate) fn set_four_bar(&self, four_bar: FourBar) {
        let mut proj = self.0.lock().unwrap();
        proj.lazy = false;
        proj.four_bar = four_bar;
    }

    pub(crate) fn name(&self) -> String {
        match &self.0.lock().unwrap().path {
            Some(path) => Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            None => "untitled".to_string(),
        }
    }

    pub(crate) fn save(&self) {
        let proj = self.0.lock().unwrap();
        let s = ron::to_string(&proj.four_bar).unwrap();
        match &proj.path {
            Some(path) => IoCtx::save(&s, path),
            None => IoCtx::save_ask(&s, "four_bar.ron", "Rusty Object Notation", &["ron"]),
        }
    }

    pub(crate) fn four_bar_ui(&self, ui: &mut Ui, interval: f64) {
        let fb = &mut self.0.lock().unwrap().four_bar;
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
            ui.add(unit("X Offset: ", &mut fb.p0.0, interval));
            ui.add(unit("Y Offset: ", &mut fb.p0.1, interval));
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
    }

    pub(crate) fn curve_io(&self, ui: &mut Ui, pivot: &mut Pivot, n: usize) {
        if ui.button("💾 Save Curve").clicked() {
            let m = Mechanism::four_bar(&self.0.lock().unwrap().four_bar);
            let curve = m.four_bar_loop_all(0., n);
            let p = match pivot {
                Pivot::Driver => &curve[0],
                Pivot::Follower => &curve[1],
                Pivot::Coupler => &curve[2],
            };
            let name = "curve.csv";
            let s = dump_csv(p).unwrap();
            IoCtx::save_ask(&s, name, "Delimiter-Separated Values", &["csv", "txt"]);
        }
        ui.selectable_value(pivot, Pivot::Coupler, "Coupler");
        ui.selectable_value(pivot, Pivot::Driver, "Driver");
        ui.selectable_value(pivot, Pivot::Follower, "Follower");
    }

    fn plot(&self, ui: &mut PlotUi, i: usize, id: usize, angle: f64, n: usize) {
        let is_main = i == id;
        let mut joints = [[0.; 2]; 5];
        let m = Mechanism::four_bar(&self.0.lock().unwrap().four_bar);
        m.apply(angle, [0, 1, 2, 3, 4], &mut joints);
        draw_link(ui, &[joints[0], joints[2]], is_main);
        draw_link(ui, &[joints[1], joints[3]], is_main);
        draw_link(ui, &joints[2..], is_main);
        draw_joints(ui, &joints);
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

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Projects {
    list: Vec<Project>,
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

    pub(crate) fn push_lazy(&mut self) -> Project {
        let lazy = Project::lazy();
        let lazy_new = lazy.clone();
        self.list.push(lazy);
        lazy_new
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, interval: f64, n: usize) {
        #[cfg(not(target_arch = "wasm32"))]
        if let [file] = &ui.ctx().input().raw.dropped_files[..] {
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
                let lazy = self.push_lazy();
                IoCtx::open("Rusty Object Notation", &["ron"], move |path, s| {
                    if let Ok(fb) = ron::from_str(&s) {
                        lazy.set_proj(Some(path), fb);
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
        if self.is_empty() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            for (i, proj) in self.list.iter().enumerate() {
                ui.selectable_value(&mut self.current, i, proj.name());
            }
        });
        ui.group(|ui| {
            let proj = &mut self.list[self.current];
            proj.four_bar_ui(ui, interval);
            ui.horizontal(|ui| proj.curve_io(ui, &mut self.pivot, n));
        });
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
