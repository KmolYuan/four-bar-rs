use super::{
    proj::{Project, Projects, Queue},
    widgets::{link, unit},
};
use eframe::egui::*;
use serde::{Deserialize, Serialize};

/// Linkage data.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Linkages {
    config: Config,
    pub projects: Projects,
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
    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("Linkages");
        self.projects
            .show(ui, self.config.interval, self.config.res);
    }

    pub fn option(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Options");
            reset_button(ui, &mut self.config);
        });
        link(ui, "Drag interval: ", &mut self.config.interval, 0.01);
        unit(ui, "Curve resolution: ", &mut self.config.res, 1);
        ui.separator();
        ui.heading("Control Tips");
        ui.label("Pan move: Left-drag / Drag");
        ui.label("Zoom: Ctrl+wheel / Pinch+stretch");
        ui.label("Box Zoom: Right-drag");
        ui.label("Reset: Right-click / Double-click");
    }

    pub fn plot(&mut self, ui: &mut plot::PlotUi) {
        self.projects.plot(ui, self.config.res);
    }

    pub fn open_project(&mut self, files: Vec<String>) {
        self.projects.iter().for_each(Project::re_open);
        for file in files {
            self.projects.open(file);
        }
    }

    pub fn current_curve(&self) -> Vec<[f64; 2]> {
        self.projects.current_curve(self.config.res)
    }

    #[allow(unused)]
    pub fn queue(&self) -> Queue {
        self.projects.queue()
    }
}
