use super::{
    proj::{Project, Projects},
    widgets::{link, unit},
};
use eframe::egui::*;
use serde::{Deserialize, Serialize};

/// Linkage data.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Linkages {
    pub(crate) cfg: Cfg,
    pub(crate) projs: Projects,
}

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub(crate) struct Cfg {
    // interval
    pub(crate) int: f64,
    // resolution
    pub(crate) res: usize,
    // Plot option - `stroke`
    pub(crate) plot_stroke: u32,
    // Plot option - `use_dot`
    pub(crate) plot_dot: bool,
}

impl Default for Cfg {
    fn default() -> Self {
        Self { int: 1., res: 360, plot_stroke: 5, plot_dot: false }
    }
}

impl Linkages {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.heading("Linkages");
        self.projs.show(ui, &self.cfg);
    }

    pub(crate) fn option(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Options");
            reset_button(ui, &mut self.cfg);
        });
        link(ui, "Drag interval: ", &mut self.cfg.int, 0.01);
        if unit(ui, "Curve resolution: ", &mut self.cfg.res, 1).changed() {
            self.projs.request_cache();
        }
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label("Theme");
                let mut vis = ui.visuals().clone();
                ui.selectable_value(&mut vis, Visuals::light(), "☀ Light");
                ui.selectable_value(&mut vis, Visuals::dark(), "🌜 Dark");
                ui.ctx().set_visuals(vis);
            });
        });
        unit(ui, "Stroke in plots: ", &mut self.cfg.plot_stroke, 1);
        ui.checkbox(&mut self.cfg.plot_dot, "Use dot curve in plots");
        ui.separator();
        ui.heading("Control Tips");
        ui.label("Pan move: Left-drag / Drag");
        ui.label("Zoom: Ctrl+wheel / Pinch+stretch");
        ui.label("Box Zoom: Right-drag");
        ui.label("Reset: Double-click");
    }

    pub(crate) fn plot(&self, ui: &mut plot::PlotUi) {
        self.projs.plot(ui);
    }

    pub(crate) fn pre_open_proj(&mut self, files: Vec<std::path::PathBuf>, res: usize) {
        self.projs.iter().for_each(Project::pre_open);
        files.into_iter().for_each(|file| self.projs.pre_open(file));
        if self.projs.is_empty() {
            self.projs.push_example();
        } else {
            self.projs.iter().for_each(|proj| proj.cache(res));
        }
    }

    pub(crate) fn poll(&mut self, ctx: &Context) {
        self.projs.poll(ctx, self.cfg.res);
    }
}
