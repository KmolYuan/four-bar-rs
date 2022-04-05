use super::{
    project::Projects,
    widgets::{angle, link, unit},
};
use crate::app::project::Queue;
use eframe::egui::{plot::PlotUi, reset_button, Ui};
use serde::{Deserialize, Serialize};

/// Linkage data.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Linkages {
    config: Config,
    driver: Driver,
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

#[derive(Deserialize, Serialize, Default, PartialEq)]
#[serde(default)]
struct Driver {
    angle: f64,
    #[serde(skip)]
    speed: f64,
}

impl Linkages {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Linkages");
            ui.collapsing("Options", |ui| {
                reset_button(ui, &mut self.config);
                ui.add(link("UI value interval: ", &mut self.config.interval, 0.01));
                ui.add(unit("Curve resolution: ", &mut self.config.res, 1));
            });
            self.projects
                .show(ui, self.config.interval, self.config.res);
        });
        ui.group(|ui| {
            ui.heading("Driver");
            reset_button(ui, &mut self.driver);
            angle(ui, "Speed: ", &mut self.driver.speed, "/s");
            angle(ui, "Angle: ", &mut self.driver.angle, "");
        });
    }

    pub fn plot(&mut self, ui: &mut PlotUi) {
        self.projects.plot(ui, self.driver.angle, self.config.res);
        if self.driver.speed != 0. {
            self.driver.angle += self.driver.speed / 60.;
            ui.ctx().request_repaint();
        }
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
