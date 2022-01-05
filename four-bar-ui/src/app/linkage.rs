use super::{
    canvas::Canvas,
    synthesis::Synthesis,
    widgets::{angle, link, unit},
    IoCtx,
};
use eframe::egui::{
    plot::{Legend, Plot},
    reset_button, Button, Ui,
};
use four_bar::FourBar;
use ron::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Linkage data.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Linkage {
    config: Config,
    driver: Driver,
    four_bar: Arc<RwLock<FourBar>>,
    #[serde(skip)]
    canvas: Canvas,
    synthesis: Synthesis,
}

impl PartialEq for Linkage {
    fn eq(&self, other: &Self) -> bool {
        self.driver == other.driver
            && *self.four_bar.read().unwrap() == *other.four_bar.read().unwrap()
    }
}

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(default)]
struct Config {
    interval: f64,
    curve_n: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval: 1.,
            curve_n: 360,
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
    pub(crate) fn open(file: Option<String>) -> Self {
        Self {
            four_bar: match file {
                #[cfg(not(target_arch = "wasm32"))]
                Some(file) => {
                    let s = std::fs::read_to_string(file).expect("Read file error");
                    Arc::new(RwLock::new(from_str(&s).expect("Deserialize error")))
                }
                None => Default::default(),
                #[cfg(target_arch = "wasm32")]
                _ => unreachable!(),
            },
            ..Self::default()
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui, ctx: &IoCtx) {
        self.canvas.update(
            self.four_bar.clone(),
            self.driver.angle,
            self.config.curve_n,
        );
        ui.group(|ui| {
            ui.heading("File");
            ui.horizontal(|ui| self.file_io(ui));
            ui.horizontal(|ui| self.canvas.curve_io(ui));
            ui.collapsing("Options", |ui| {
                reset_button(ui, &mut self.config);
                ui.add(link("UI value interval: ", &mut self.config.interval, 0.01));
                ui.add(unit("Curve resolution: ", &mut self.config.curve_n, 1));
            });
        });
        ui.group(|ui| {
            ui.heading("Dimension");
            reset_button(ui, &mut *self.four_bar.write().unwrap());
            self.parameter(ui);
        });
        ui.group(|ui| {
            ui.heading("Driver");
            reset_button(ui, &mut self.driver);
            angle(ui, "Speed: ", &mut self.driver.speed, "/s");
            angle(ui, "Angle: ", &mut self.driver.angle, "");
        });
        ui.group(|ui| self.synthesis.ui(ui, ctx, self.four_bar.clone()));
    }

    fn file_io(&mut self, ui: &mut Ui) {
        if ui.button("💾 Save").clicked() {
            let name = "four_bar.ron";
            let s = to_string(&*self.four_bar.read().unwrap()).unwrap();
            IoCtx::save(&s, name, "Rusty Object Notation", &["ron"]);
        }
        if ui.button("🖴 Open").clicked() {
            let four_bar = self.four_bar.clone();
            IoCtx::open("Rusty Object Notation", &["ron"], move |s| {
                if let Ok(fb) = from_str(&s) {
                    *four_bar.write().unwrap() = fb;
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let [file] = &ui.ctx().input().raw.dropped_files[..] {
            if let Some(path) = &file.path {
                let s = std::fs::read_to_string(path).unwrap_or_default();
                if let Ok(fb) = from_str(&s) {
                    *self.four_bar.write().unwrap() = fb;
                }
            }
        }
    }

    fn parameter(&mut self, ui: &mut Ui) {
        let n = self.config.interval;
        if ui.button("Normalize").clicked() {
            self.four_bar.write().unwrap().normalize();
        }
        let mut four_bar = self.four_bar.write().unwrap();
        ui.group(|ui| {
            ui.heading("Offset");
            if ui
                .add_enabled(!four_bar.is_aligned(), Button::new("Reset"))
                .clicked()
            {
                four_bar.align();
            }
            ui.add(unit("X Offset: ", &mut four_bar.p0.0, n));
            ui.add(unit("Y Offset: ", &mut four_bar.p0.1, n));
            angle(ui, "Rotation: ", &mut four_bar.a, "");
        });
        ui.group(|ui| {
            ui.heading("Parameters");
            ui.add(link("Ground: ", &mut four_bar.l0, n));
            ui.add(link("Driver: ", &mut four_bar.l1, n));
            ui.add(link("Coupler: ", &mut four_bar.l2, n));
            ui.add(link("Follower: ", &mut four_bar.l3, n));
            ui.checkbox(&mut four_bar.inv, "Invert follower and coupler");
        });
        ui.group(|ui| {
            ui.heading("Coupler");
            ui.add(link("Extended: ", &mut four_bar.l4, n));
            angle(ui, "Angle: ", &mut four_bar.g, "");
        });
    }

    pub(crate) fn plot(&mut self, ui: &mut Ui) {
        Plot::new("canvas")
            .data_aspect(1.)
            .legend(Legend::default())
            .show(ui, |ui| {
                self.canvas.ui(ui);
                self.synthesis.plot(ui);
            });
        if self.driver.speed != 0. {
            self.driver.angle += self.driver.speed / 60.;
            ui.ctx().request_repaint();
        }
    }
}
