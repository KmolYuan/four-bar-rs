use super::widgets::*;
use eframe::egui::*;
use serde::{Deserialize, Serialize};

/// Linkage data.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Linkages {
    pub(crate) cfg: Cfg,
    pub(crate) projs: super::proj::Projects,
}

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub(crate) struct Cfg {
    // interval
    pub(crate) int: f64,
    // resolution
    pub(crate) res: usize,
    #[cfg(target_arch = "wasm32")]
    pub(crate) web_data: bool,
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            int: 1.,
            res: 360,
            #[cfg(target_arch = "wasm32")]
            web_data: false,
        }
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
        nonzero_f(ui, "Drag interval: ", &mut self.cfg.int, 0.01);
        if nonzero_i(ui, "Curve resolution: ", &mut self.cfg.res, 1).changed() {
            self.projs.request_cache();
        }
        #[cfg(target_arch = "wasm32")]
        if ui
            .checkbox(&mut self.cfg.web_data, "Save local data")
            .clicked()
            && !self.cfg.web_data
        {
            #[wasm_bindgen::prelude::wasm_bindgen]
            extern "C" {
                fn clear_storage();
            }
            clear_storage();
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

    pub(crate) fn preload(&mut self, files: Vec<std::path::PathBuf>, res: usize) {
        self.projs.preload(files, res);
    }

    pub(crate) fn poll(&mut self, ctx: &Context) {
        self.projs.poll(ctx, self.cfg.res);
    }
}
