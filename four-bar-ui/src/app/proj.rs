use self::impl_proj::*;
use super::{link::Cfg, widgets::*};
use crate::io;
use eframe::egui::*;
use four_bar::{FourBar, SFourBar};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

mod fb_ui;
mod impl_proj;
mod undo;

#[derive(Default, Deserialize, Serialize, PartialEq, Eq, Copy, Clone)]
pub(crate) enum Pivot {
    Driver,
    Follower,
    #[default]
    Coupler,
}

impl Pivot {
    const fn name(&self) -> &'static str {
        match self {
            Pivot::Driver => "Driver",
            Pivot::Follower => "Follower",
            Pivot::Coupler => "Coupler",
        }
    }
}

#[derive(Default, Clone)]
pub(crate) struct Queue(Arc<mutex::RwLock<Vec<Project>>>);

impl Queue {
    pub(crate) fn push(&self, path: Option<PathBuf>, fb: io::Fb) {
        self.0.write().push(Project::new(path, fb));
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Projects {
    curr: usize,
    list: Vec<Project>,
    #[serde(skip)]
    pivot: Pivot,
    #[serde(skip)]
    queue: Queue,
    #[serde(skip)]
    path: io::Cache<PathBuf>,
}

impl Projects {
    pub(crate) fn preload(&mut self, files: Vec<PathBuf>, res: usize) {
        files.into_iter().for_each(|p| self.pre_open(p));
        self.list.iter_mut().for_each(|p| p.preload());
        if self.list.is_empty() && self.queue.0.read().is_empty() {
            self.push_fb_example();
        } else {
            self.list.iter_mut().for_each(|p| p.cache(res));
        }
        // Current index boundary check
        if !self.list.is_empty() && self.curr >= self.list.len() {
            self.curr = self.list.len() - 1;
        }
    }

    fn push_fb_example(&self) {
        self.queue.push(None, io::Fb::Fb(FourBar::example()));
    }

    fn push_sfb_example(&self) {
        self.queue.push(None, io::Fb::SFb(SFourBar::example()));
    }

    fn pre_open(&mut self, path: PathBuf) {
        // Check duplicates
        if self.list.iter().all(|proj| proj.path() != Some(&path)) {
            if let Some(proj) = Project::pre_open(path) {
                self.list.push(proj);
            }
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
            if let Some(p) = self.list.get_mut(self.curr) {
                p.set_path(path);
            }
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
        ui.horizontal(|ui| {
            if self.list.is_empty() {
                return;
            }
            self.select(ui);
            if small_btn(ui, "💾", "Save (Ctrl+S)") || hotkey!(ui, CTRL + S) {
                self.save_curr(self.list[self.curr].path().is_none());
            }
            if small_btn(ui, "💾 Save As", "Ctrl+Shift+S") || hotkey!(ui, CTRL + SHIFT + S) {
                self.save_curr(true);
            }
            if self.list[self.curr].is_unsaved() {
                ui.menu_button("✖?", |ui| {
                    if ui.button("✖ Close Without Save").clicked() {
                        self.close_curr();
                    }
                })
                .response
                .on_hover_text("Close Options");
            } else if ui
                .small_button("✖")
                .on_hover_text("Close (Ctrl+W)")
                .clicked()
                || hotkey!(ui, CTRL + W)
            {
                self.close_curr();
            }
        });
        if self.list.is_empty() {
            ui.heading("No project here!");
            ui.label("Please open or create a project.");
        } else {
            self.list[self.curr].show(ui, &mut self.pivot, cfg);
        }
    }

    fn save_curr(&mut self, ask: bool) {
        let proj = &self.list[self.curr];
        let (_, fb) = proj.fb_state();
        match proj.path() {
            Some(path) if !ask => io::save_ron(&fb, path),
            _ => {
                let path = self.path.clone();
                io::save_ron_ask(&fb, &proj.name(), move |p| *path.borrow_mut() = Some(p));
            }
        }
        self.list[self.curr].mark_saved();
    }

    pub(crate) fn select(&mut self, ui: &mut Ui) {
        if self.list.is_empty() {
            ComboBox::from_id_source("proj").show_ui(ui, |_| ());
        } else {
            ComboBox::from_id_source("proj").show_index(ui, &mut self.curr, self.list.len(), |i| {
                let proj = &self.list[i];
                if proj.is_unsaved() {
                    proj.proj_name() + "*"
                } else {
                    proj.proj_name()
                }
            });
        }
    }

    fn close_curr(&mut self) {
        self.list.remove(self.curr);
        if self.curr > 0 {
            self.curr -= 1;
        }
    }

    pub(crate) fn current_fb_state(&self) -> Option<(f64, io::Fb)> {
        Some(self.list.get(self.curr)?.fb_state())
    }

    pub(crate) fn current_curve(&self) -> Option<io::Curve> {
        Some(self.list.get(self.curr)?.curve())
    }

    pub(crate) fn current_sphere(&self) -> Option<[f64; 4]> {
        self.list.get(self.curr)?.get_sphere()
    }

    pub(crate) fn request_cache(&mut self) {
        if let Some(p) = self.list.get_mut(self.curr) {
            p.request_cache();
        }
    }

    pub(crate) fn plot(&self, ui: &mut egui_plot::PlotUi) {
        for (i, proj) in self.list.iter().enumerate() {
            proj.plot(ui, i, self.curr);
        }
    }
}
