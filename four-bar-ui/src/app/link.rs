use eframe::egui::*;
use serde::{Deserialize, Serialize};

/// Linkage data.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Linkages {
    pub(crate) projs: super::proj::Projects,
}

impl Linkages {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.heading("Linkages");
        self.projs.show(ui);
    }

    pub(crate) fn plot(&self, ui: &mut egui_plot::PlotUi) {
        self.projs.plot(ui);
    }

    pub(crate) fn preload(&mut self, files: Vec<std::path::PathBuf>) {
        self.projs.preload(files);
    }

    pub(crate) fn poll(&mut self, ctx: &Context) {
        self.projs.poll(ctx);
    }
}
