use eframe::egui::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct BluePrint {
    img: Option<PathBuf>,
}

impl BluePrint {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.heading("Blue Print");
        ui.label("The tool let you reference designs from pictures.");
        if let Some(path) = &self.img {
            ui.horizontal(|ui| {
                ui.label("🖻");
                ui.label(path.to_string_lossy());
            });
        } else {
            ui.colored_label(Color32::RED, "No image yet.");
        }
    }
}
