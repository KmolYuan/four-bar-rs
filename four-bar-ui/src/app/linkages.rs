use super::{
    project::{Projects, Queue},
    widgets::{link, unit},
};
use eframe::egui::*;
use serde::{Deserialize, Serialize};

/// Linkage data.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Linkages {
    config: Config,
    projects: Projects,
}

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(default)]
struct Config {
    interval: f64,
    res: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval: 1.,
            res: 360,
        }
    }
}

impl Linkages {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Linkages");
            self.projects
                .show(ui, self.config.interval, self.config.res);
        });
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Options");
                reset_button(ui, &mut self.config);
            });
            ui.add(link("Drag interval: ", &mut self.config.interval, 0.01));
            ui.add(unit("Curve resolution: ", &mut self.config.res, 1));
        });
    }

    pub fn plot(&mut self, ui: &mut plot::PlotUi) {
        self.projects.plot(ui);
    }

    pub fn open_project(&mut self, files: Vec<String>) {
        self.projects.reload();
        for file in files {
            self.projects.open(file);
        }
    }

    pub fn select_projects(&mut self, ui: &mut Ui) -> bool {
        self.projects.select(ui)
    }

    pub fn current_curve(&self) -> Vec<[f64; 2]> {
        self.projects.current_curve(self.config.res)
    }

    pub fn queue(&self) -> Queue {
        self.projects.queue()
    }
}
