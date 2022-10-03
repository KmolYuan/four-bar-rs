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
    config: Config,
    pub(crate) projs: Projects,
}

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(default)]
struct Config {
    interval: f64,
    res: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { interval: 1., res: 360 }
    }
}

impl Linkages {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.heading("Linkages");
        self.projs.show(ui, self.config.interval, self.config.res);
    }

    pub(crate) fn option(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Options");
            reset_button(ui, &mut self.config);
        });
        link(ui, "Drag interval: ", &mut self.config.interval, 0.01);
        if unit(ui, "Curve resolution: ", &mut self.config.res, 1).changed() {
            self.projs.request_cache();
        }
        let mut vis = ui.visuals().clone();
        vis.light_dark_radio_buttons(ui);
        ui.ctx().set_visuals(vis);
        ui.separator();
        ui.heading("Control Tips");
        ui.label("Pan move: Left-drag / Drag");
        ui.label("Zoom: Ctrl+wheel / Pinch+stretch");
        ui.label("Box Zoom: Right-drag");
        ui.label("Reset: Right-click / Double-click");
    }

    pub(crate) fn plot(&self, ui: &mut plot::PlotUi) {
        self.projs.plot(ui);
    }

    pub(crate) fn pre_open_proj(&mut self, files: Vec<std::path::PathBuf>) {
        self.projs.iter().for_each(Project::pre_open);
        files.into_iter().for_each(|file| self.projs.pre_open(file));
        if self.projs.is_empty() {
            self.projs.push_example();
        }
    }

    pub(crate) fn poll(&mut self, ctx: &Context) {
        self.projs.poll(ctx, self.config.res);
    }
}
