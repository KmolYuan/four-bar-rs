use self::switch::*;
use super::{link::Cfg, widgets::*};
use crate::io;
use eframe::egui::*;
use four_bar::{CurveGen as _, *};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

mod switch;
mod ui;
mod undo;

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

#[derive(Default, Clone)]
pub(crate) struct Queue(Arc<mutex::RwLock<Vec<ProjSwitch>>>);

impl Queue {
    pub(crate) fn push(&self, path: Option<PathBuf>, fb: io::Fb) {
        self.0.write().push(ProjSwitch::new(path, fb));
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Projects {
    curr: usize,
    pivot: Pivot,
    list: Vec<ProjSwitch>,
    #[serde(skip)]
    queue: Queue,
    #[serde(skip)]
    path: Rc<RefCell<Option<PathBuf>>>,
}

impl Projects {
    pub(crate) fn preload(&mut self, files: Vec<PathBuf>, res: usize) {
        files.into_iter().for_each(|p| self.pre_open(p));
        self.list.iter_mut().for_each(|p| p.preload());
        self.list.retain(|p| p.path().is_some());
        if self.list.is_empty() && self.queue.0.read().is_empty() {
            self.push_fb_example();
        } else {
            self.list.iter_mut().for_each(|p| p.cache(res));
        }
    }

    fn push_fb_example(&self) {
        self.queue.push(None, io::Fb::Fb(FourBar::example()));
    }

    fn push_sfb_example(&self) {
        self.queue.push(None, io::Fb::SFb(SFourBar::example()));
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
        let len = self.queue.0.read().len();
        if len > 0 {
            self.list.reserve(len);
            while let Some(mut proj) = self.queue.0.write().pop() {
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
            if ui.button("✚ Planar").clicked() {
                self.push_fb_example();
            }
            if ui.button("✚ Spherical").clicked() {
                self.push_sfb_example();
            }
        });
        ui.separator();
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
            ComboBox::from_label("").show_index(ui, &mut self.curr, self.list.len(), |i| {
                let proj = &self.list[i];
                match proj {
                    ProjSwitch::Fb(_) => format!("[P] {}", proj.name()),
                    ProjSwitch::SFb(_) => format!("[S] {}", proj.name()),
                }
            });
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

    pub(crate) fn current_curve(&self) -> io::Curve {
        self.list[self.curr].curve()
    }

    pub(crate) fn current_sphere(&self) -> Option<[f64; 4]> {
        self.list.get(self.curr)?.get_sphere()
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
