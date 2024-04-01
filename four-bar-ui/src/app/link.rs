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

    pub(crate) fn option(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label("Theme");
                let mut vis = ui.visuals().clone();
                ui.selectable_value(&mut vis, Visuals::light(), "☀ Light");
                ui.selectable_value(&mut vis, Visuals::dark(), "🌜 Dark");
                ui.ctx().set_visuals(vis);
            });
        });
        ui.separator();
        ui.heading("Control Tips");
        ui.label("Pan move: Left-drag / Drag");
        ui.label("Zoom: Ctrl+wheel / Pinch+stretch");
        ui.label("Box Zoom: Right-drag");
        ui.label("Reset: Double-click");
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
