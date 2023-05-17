use self::proj_inner::*;
use super::{
    io::{self, Fb},
    link::Cfg,
    widgets::*,
};
use eframe::egui::*;
use four_bar::{CurveGen as _, *};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, RwLock},
};

mod proj_inner;
mod undo;

const JOINT_COLOR: Color32 = Color32::from_rgb(93, 69, 56);
const LINK_COLOR: Color32 = Color32::from_rgb(165, 151, 132);

fn draw_link(ui: &mut plot::PlotUi, points: &[[f64; 2]], is_main: bool) {
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

fn angle_bound_btns(ui: &mut Ui, theta2: &mut f64, start: f64, end: f64) -> Response {
    ui.group(|ui| {
        fn copy_btn(ui: &mut Ui, start: f64, end: f64, suffix: &str) {
            ui.horizontal(|ui| {
                let s_str = format!("{start:.04}");
                if ui.selectable_label(false, &s_str).clicked() {
                    ui.output_mut(|s| s.copied_text = s_str);
                }
                let e_str = format!("{end:.04}");
                if ui.selectable_label(false, &e_str).clicked() {
                    ui.output_mut(|s| s.copied_text = e_str);
                }
                ui.label(suffix);
            });
        }
        ui.label("Click to copy angle bounds:");
        copy_btn(ui, start, end, "rad");
        copy_btn(ui, start.to_degrees(), end.to_degrees(), "deg");
        ui.horizontal(|ui| {
            let mut res1 = ui.button("➡ To Start");
            if res1.clicked() {
                res1.mark_changed();
                *theta2 = start;
            }
            let mut res2 = ui.button("➡ To End");
            if res2.clicked() {
                res2.mark_changed();
                *theta2 = end;
            }
            res1 | res2
        })
        .inner
    })
    .inner
}

#[derive(Default, Deserialize, Serialize, PartialEq, Eq, Copy, Clone)]
pub(crate) enum Pivot {
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
pub(crate) struct Queue(Arc<RwLock<Vec<ProjSwitch>>>);

impl Queue {
    pub(crate) fn push(&self, path: Option<PathBuf>, fb: Fb) {
        self.0.write().unwrap().push(ProjSwitch::new(path, fb));
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Projects {
    path: Rc<RefCell<Option<PathBuf>>>,
    list: Vec<ProjSwitch>,
    queue: Queue,
    pivot: Pivot,
    curr: usize,
}

impl Projects {
    pub(crate) fn preload(&mut self, files: Vec<PathBuf>, res: usize) {
        files.into_iter().for_each(|p| self.pre_open(p));
        self.list.iter_mut().for_each(|p| p.preload());
        self.list.retain(|p| p.path().is_some());
        if self.list.is_empty() {
            self.push_fb_example();
        } else {
            self.list.iter_mut().for_each(|p| p.cache(res));
        }
    }

    pub(crate) fn push_fb_example(&self) {
        self.queue.0.write().unwrap().push(ProjSwitch::new_fb());
    }

    pub(crate) fn push_sfb_example(&self) {
        self.queue.0.write().unwrap().push(ProjSwitch::new_sfb());
    }

    pub(crate) fn pre_open(&mut self, path: PathBuf) {
        if self.list.iter().any(|proj| proj.path() == Some(&path)) {
            return;
        }
        if let Some(proj) = ProjSwitch::pre_open(path) {
            self.list.push(proj);
        }
    }

    pub(crate) fn queue(&self) -> Queue {
        self.queue.clone()
    }

    pub(crate) fn poll(&mut self, ctx: &Context, n: usize) {
        #[cfg(not(target_arch = "wasm32"))]
        ctx.input(|s| {
            for file in s.raw.dropped_files.iter() {
                if let Some(path) = &file.path {
                    self.pre_open(path.clone());
                }
            }
        });
        let len = self.queue.0.read().unwrap().len();
        if len > 0 {
            self.list.reserve(len);
            while let Some(mut proj) = self.queue.0.write().unwrap().pop() {
                proj.cache(n);
                self.list.push(proj);
            }
            self.curr = self.list.len() - 1;
            ctx.request_repaint();
        }
        if let Some(path) = self.path.borrow_mut().take() {
            self.list[self.curr].set_path(path);
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, cfg: &Cfg) {
        ui.horizontal(|ui| {
            if ui.button("🖴 Load").clicked() || hotkey!(ui, CTRL + O) {
                let q = self.queue();
                io::open_ron(move |path, fb| q.push(Some(path), fb));
            }
            if ui.button("🗋 New Planar").clicked() {
                self.push_fb_example();
            }
            if ui.button("🗋 New Spherical").clicked() {
                self.push_sfb_example();
            }
        });
        if self.select(ui, true) {
            self.list[self.curr].show(ui, &mut self.pivot, cfg);
        } else {
            ui.heading("No project here!");
            ui.label("Please open or create a project.");
        }
    }

    pub(crate) fn select(&mut self, ui: &mut Ui, show_btn: bool) -> bool {
        if self.list.is_empty() {
            return false;
        }
        ui.horizontal(|ui| {
            ComboBox::from_label("")
                .show_index(ui, &mut self.curr, self.list.len(), |i| self.list[i].name());
            if !show_btn {
                return;
            }
            if small_btn(ui, "💾", "Save (Ctrl+S)") || hotkey!(ui, CTRL + S) {
                let proj = &self.list[self.curr];
                let (_, fb) = proj.fb_state();
                if let Some(path) = proj.path() {
                    io::save_ron(&fb, path);
                } else {
                    let path = self.path.clone();
                    io::save_ron_ask(&fb, &proj.name(), move |p| _ = path.borrow_mut().replace(p));
                }
            }
            if small_btn(ui, "💾 Save As", "Ctrl+Shift+S") || hotkey!(ui, CTRL + SHIFT + S) {
                let proj = &self.list[self.curr];
                let (_, fb) = proj.fb_state();
                let path = self.path.clone();
                io::save_ron_ask(&fb, &proj.name(), move |p| _ = path.borrow_mut().replace(p));
            }
            if small_btn(ui, "✖", "Close (Ctrl+W)") || hotkey!(ui, CTRL + W) {
                self.list.remove(self.curr);
                if self.curr > 0 {
                    self.curr -= 1;
                }
            }
        });
        !self.list.is_empty()
    }

    pub(crate) fn current_fb_state(&self) -> (f64, io::Fb) {
        self.list[self.curr].fb_state()
    }

    pub(crate) fn current_curve(&self) -> super::plotter::Curve {
        self.list[self.curr].curve()
    }

    pub(crate) fn request_cache(&mut self) {
        self.list[self.curr].request_cache();
    }

    pub(crate) fn plot(&self, ui: &mut plot::PlotUi) {
        for (i, proj) in self.list.iter().enumerate() {
            proj.plot(ui, i, self.curr);
        }
    }
}
