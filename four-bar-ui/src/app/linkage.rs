use super::{synthesis::Synthesis, IoCtx};
use crate::{as_values::as_values, csv_io::dump_csv};
use eframe::egui::{
    plot::{Legend, Line, Plot, Points, Polygon},
    reset_button, Button, CentralPanel, Color32, CtxRef, DragValue, Ui,
};
use four_bar::{FourBar, Mechanism};
use ron::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::{
    f64::consts::TAU,
    sync::{Arc, RwLock},
};

macro_rules! unit {
    ($label:literal, $attr:expr, $inter:expr, $ui:ident) => {
        $ui.add(DragValue::new(&mut $attr).prefix($label).speed($inter));
    };
}

macro_rules! link {
    ($label:literal, $attr:expr, $inter:expr, $ui:ident) => {
        $ui.add(
            DragValue::new(&mut $attr)
                .prefix($label)
                .clamp_range(0.0001..=9999.)
                .speed($inter),
        );
    };
}

macro_rules! angle {
    ($label:literal, $attr:expr, $ui:ident, $t:literal) => {
        $ui.horizontal(|ui| {
            if $attr < 0. {
                $attr += TAU;
            }
            let mut deg = $attr.to_degrees();
            if ui
                .add(
                    DragValue::new(&mut deg)
                        .prefix($label)
                        .suffix(concat![" deg", $t])
                        .clamp_range(0..=360)
                        .speed(1.),
                )
                .changed()
            {
                $attr = deg.to_radians();
            }
            ui.add(
                DragValue::new(&mut $attr)
                    .suffix(concat![" rad", $t])
                    .min_decimals(2)
                    .clamp_range((0.)..=TAU)
                    .speed(0.01),
            );
        });
    };
    ($label:literal, $attr:expr, $ui:ident) => {
        if TAU - $attr < 1e-20 {
            $attr = 0.;
        }
        angle!($label, $attr, $ui, "");
    };
}

macro_rules! num {
    ($label:literal, $attr:expr, $inter:expr, $min:expr, $ui:ident) => {
        $ui.add(
            DragValue::new(&mut $attr)
                .prefix($label)
                .clamp_range($min..=9999)
                .speed($inter),
        );
    };
}

macro_rules! draw_link {
    ($a:expr, $b:expr) => {
        Line::new(as_values(&[$a, $b]))
            .width(3.)
            .color(Color32::from_rgb(165, 151, 132))
    };
    ($a:expr, $b:expr $(, $c:expr)+) => {
        Polygon::new(as_values(&[$a, $b $(, $c)+]))
            .width(3.)
            .fill_alpha(0.6)
            .color(Color32::from_rgb(165, 151, 132))
    };
}

macro_rules! draw_path {
    ($name:literal, $path:expr) => {
        Line::new(as_values(&$path)).name($name).width(3.)
    };
}

#[derive(Deserialize, Serialize, PartialEq)]
enum Pivot {
    Driver,
    Follower,
    Coupler,
}

impl Default for Pivot {
    fn default() -> Self {
        Self::Coupler
    }
}

/// Linkage data.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Linkage {
    config: Config,
    driver: Driver,
    four_bar: Arc<RwLock<FourBar>>,
    inv_coupler: bool,
    path1: Vec<[f64; 2]>,
    path2: Vec<[f64; 2]>,
    path3: Vec<[f64; 2]>,
    joints: [[f64; 2]; 5],
    pivot: Pivot,
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
    drive: f64,
    speed: f64,
}

impl Linkage {
    fn update_mechanism(&mut self) {
        let m = Mechanism::four_bar(&*self.four_bar.read().unwrap());
        m.apply(self.driver.drive, [0, 1, 2, 3, 4], &mut self.joints);
        let [path1, path2, path3] = m.four_bar_loop_all(0., self.config.curve_n);
        self.path1 = path1;
        self.path2 = path2;
        self.path3 = path3;
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui, ctx: &IoCtx) {
        self.update_mechanism();
        ui.group(|ui| {
            ui.heading("File");
            ui.horizontal(|ui| self.file_io(ui, ctx));
            ui.horizontal(|ui| self.curve_io(ui, ctx));
            ui.collapsing("Options", |ui| {
                reset_button(ui, &mut self.config);
                link!("UI value interval: ", self.config.interval, 0.01, ui);
                num!("Number of curve points: ", self.config.curve_n, 1, 10, ui);
            });
        });
        ui.group(|ui| {
            ui.heading("Dimension");
            reset_button(ui, self);
            self.parameter(ui);
        });
        ui.group(|ui| {
            ui.heading("Driver");
            reset_button(ui, &mut self.driver);
            angle!("Speed: ", self.driver.speed, ui, "/s");
            angle!("Angle: ", self.driver.drive, ui);
        });
        ui.group(|ui| self.synthesis.ui(ui, ctx, self.four_bar.clone()));
    }

    fn file_io(&mut self, ui: &mut Ui, ctx: &IoCtx) {
        if ui.button("💾 Save").clicked() {
            let name = "four_bar.ron";
            let s = to_string(&*self.four_bar.read().unwrap()).unwrap();
            #[cfg(target_arch = "wasm32")]
            let _ = ctx.save(&s, name);
            #[cfg(not(target_arch = "wasm32"))]
            let _ = ctx.save(&s, name, "Rusty Object Notation", &["ron"]);
        }
        if ui.button("🖴 Open").clicked() {
            #[cfg(target_arch = "wasm32")]
            let _ = ctx.open(&["ron"]);
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(s) = ctx.open("Rusty Object Notation", &["ron"]) {
                if let Ok(four_bar) = from_str(s.as_str()) {
                    *self.four_bar.write().unwrap() = four_bar;
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(s) = ctx.open_result() {
            if let Ok(four_bar) = from_str(s.as_str()) {
                *self.four_bar.write().unwrap() = four_bar;
            }
        }
    }

    fn curve_io(&mut self, ui: &mut Ui, ctx: &IoCtx) {
        if ui.button("💾 Save Curve").clicked() {
            let path = match self.pivot {
                Pivot::Driver => &self.path1,
                Pivot::Follower => &self.path2,
                Pivot::Coupler => &self.path3,
            };
            let name = "curve.csv";
            let s = dump_csv(path).unwrap();
            #[cfg(target_arch = "wasm32")]
            let _ = ctx.save(&s, name);
            #[cfg(not(target_arch = "wasm32"))]
            let _ = ctx.save(&s, name, "Delimiter-Separated Values", &["csv", "txt"]);
        }
        ui.selectable_value(&mut self.pivot, Pivot::Coupler, "Coupler");
        ui.selectable_value(&mut self.pivot, Pivot::Driver, "Driver");
        ui.selectable_value(&mut self.pivot, Pivot::Follower, "Follower");
    }

    fn parameter(&mut self, ui: &mut Ui) {
        let interval = self.config.interval;
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
            unit!("X Offset: ", four_bar.p0.0, interval, ui);
            unit!("Y Offset: ", four_bar.p0.1, interval, ui);
            angle!("Rotation: ", four_bar.a, ui);
        });
        ui.group(|ui| {
            ui.heading("Parameters");
            link!("Ground: ", four_bar.l0, interval, ui);
            link!("Driver: ", four_bar.l1, interval, ui);
            link!("Coupler: ", four_bar.l2, interval, ui);
            link!("Follower: ", four_bar.l3, interval, ui);
            ui.checkbox(&mut four_bar.inv, "Invert follower and coupler");
            ui.checkbox(&mut self.inv_coupler, "Invert coupler point");
        });
        ui.group(|ui| {
            ui.heading("Coupler");
            link!("Extended: ", four_bar.l4, interval, ui);
            angle!("Angle: ", four_bar.g, ui);
        });
    }

    pub(crate) fn plot(&mut self, ctx: &CtxRef) {
        CentralPanel::default().show(ctx, |ui| {
            let mut plot = Plot::new("canvas")
                .line(draw_link![self.joints[0], self.joints[2]])
                .line(draw_link![self.joints[1], self.joints[3]])
                .polygon(draw_link![self.joints[2], self.joints[3], self.joints[4]])
                .points(
                    Points::new(as_values(&[self.joints[0], self.joints[1]]))
                        .radius(7.)
                        .color(Color32::from_rgb(93, 69, 56)),
                )
                .points(
                    Points::new(as_values(&[self.joints[2], self.joints[3], self.joints[4]]))
                        .radius(5.)
                        .color(Color32::from_rgb(128, 96, 77)),
                )
                .line(draw_path!("Crank pivot", self.path1))
                .line(draw_path!("Follower pivot", self.path2))
                .line(draw_path!("Coupler pivot", self.path3));
            if !self.synthesis.curve.is_empty() {
                plot = plot.line(draw_path!("Synthesis target", self.synthesis.curve));
            }
            ui.add(plot.data_aspect(1.).legend(Legend::default()));
            if self.driver.speed != 0. {
                self.driver.drive += self.driver.speed / 60.;
                ui.ctx().request_repaint();
            }
        });
    }
}
