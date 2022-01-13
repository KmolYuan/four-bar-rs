use super::{
    project::Projects,
    synthesis::Synthesis,
    widgets::{angle, link, unit},
    IoCtx,
};
use eframe::egui::{plot::PlotUi, reset_button, Ui};
use serde::{Deserialize, Serialize};

/// Linkage data.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Linkage {
    config: Config,
    driver: Driver,
    projects: Projects,
    synthesis: Synthesis,
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

impl Linkage {
    pub(crate) fn show(&mut self, ui: &mut Ui, ctx: &IoCtx) {
        ui.group(|ui| {
            ui.heading("Linkage");
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
        ui.group(|ui| self.synthesis.show(ui, ctx, self.projects.queue()));
    }

    pub(crate) fn plot(&mut self, ui: &mut PlotUi) {
        self.projects.plot(ui, self.driver.angle, self.config.res);
        self.synthesis.plot(ui);
        if self.driver.speed != 0. {
            self.driver.angle += self.driver.speed / 60.;
            ui.ctx().request_repaint();
        }
    }

    pub(crate) fn open_project(&mut self, file: String) {
        let s = std::fs::read_to_string(&file).expect("Read file error");
        let four_bar = ron::from_str(&s).expect("Deserialize error");
        self.projects.push(Some(file), four_bar);
    }
}
