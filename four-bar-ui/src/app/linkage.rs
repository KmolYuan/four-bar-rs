use super::{synthesis::Synthesis, IoCtx};
use crate::{as_values::as_values, csv_io::dump_csv};
use eframe::egui::{
    emath::Numeric,
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

fn unit<'a>(label: &'a str, attr: &'a mut f64, inter: f64) -> DragValue<'a> {
    DragValue::new(attr).prefix(label).speed(inter)
}

fn link<'a>(label: &'a str, attr: &'a mut f64, inter: f64) -> DragValue<'a> {
    DragValue::new(attr)
        .prefix(label)
        .clamp_range(0.0001..=f64::MAX)
        .speed(inter)
}

fn angle(ui: &mut Ui, label: &str, attr: &mut f64, suffix: &'static str) {
    if suffix.is_empty() && TAU - *attr < 1e-20 {
        *attr = 0.;
    }
    ui.horizontal(|ui| {
        if *attr < 0. {
            *attr += TAU;
        }
        let mut deg = attr.to_degrees();
        let dv = DragValue::new(&mut deg)
            .prefix(label)
            .suffix(String::from(" deg") + suffix)
            .speed(1.);
        if ui.add(dv).changed() {
            *attr = deg.to_radians();
        }
        let dv = DragValue::new(attr)
            .suffix(String::from(" rad") + suffix)
            .min_decimals(2)
            .speed(0.01);
        ui.add(dv);
    });
}

fn num<'a>(label: &'a str, attr: &'a mut impl Numeric, inter: f64, min: f64) -> DragValue<'a> {
    DragValue::new(attr)
        .prefix(label)
        .clamp_range(min..=f64::MAX)
        .speed(inter)
}

fn draw_link2(a: [f64; 2], b: [f64; 2]) -> Line {
    Line::new(as_values(&[a, b]))
        .width(3.)
        .color(Color32::from_rgb(165, 151, 132))
}

fn draw_link3(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Polygon {
    Polygon::new(as_values(&[a, b, c]))
        .width(3.)
        .fill_alpha(0.6)
        .color(Color32::from_rgb(165, 151, 132))
}

fn draw_path(name: &str, path: &[[f64; 2]]) -> Line {
    Line::new(as_values(path)).name(name).width(3.)
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
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn open(file: Option<&str>) -> Self {
        Self {
            four_bar: match file {
                Some(file) => {
                    let s = std::fs::read_to_string(file).expect("Read file error");
                    Arc::new(RwLock::new(from_str(&s).expect("Deserialize error")))
                }
                None => Default::default(),
            },
            ..Self::default()
        }
    }

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
                ui.add(link("UI value interval: ", &mut self.config.interval, 0.01));
                ui.add(num(
                    "Number of curve points: ",
                    &mut self.config.curve_n,
                    1.,
                    10.,
                ));
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
            angle(ui, "Speed: ", &mut self.driver.speed, "/s");
            angle(ui, "Angle: ", &mut self.driver.drive, "");
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
            let four_bar = self.four_bar.clone();
            ctx.open("Rusty Object Notation", &["ron"], move |s| {
                if let Ok(fb) = from_str(&s) {
                    *four_bar.write().unwrap() = fb;
                }
            });
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
            ui.add(unit("X Offset: ", &mut four_bar.p0.0, interval));
            ui.add(unit("Y Offset: ", &mut four_bar.p0.1, interval));
            angle(ui, "Rotation: ", &mut four_bar.a, "");
        });
        ui.group(|ui| {
            ui.heading("Parameters");
            ui.add(link("Ground: ", &mut four_bar.l0, interval));
            ui.add(link("Driver: ", &mut four_bar.l1, interval));
            ui.add(link("Coupler: ", &mut four_bar.l2, interval));
            ui.add(link("Follower: ", &mut four_bar.l3, interval));
            ui.checkbox(&mut four_bar.inv, "Invert follower and coupler");
        });
        ui.group(|ui| {
            ui.heading("Coupler");
            ui.add(link("Extended: ", &mut four_bar.l4, interval));
            angle(ui, "Angle: ", &mut four_bar.g, "");
        });
    }

    pub(crate) fn plot(&mut self, ctx: &CtxRef) {
        CentralPanel::default().show(ctx, |ui| {
            let mut plot = Plot::new("canvas")
                .line(draw_link2(self.joints[0], self.joints[2]))
                .line(draw_link2(self.joints[1], self.joints[3]))
                .polygon(draw_link3(self.joints[2], self.joints[3], self.joints[4]))
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
                .line(draw_path("Crank pivot", &self.path1))
                .line(draw_path("Follower pivot", &self.path2))
                .line(draw_path("Coupler pivot", &self.path3));
            if !self.synthesis.curve.is_empty() {
                plot = plot.line(draw_path("Synthesis target", &self.synthesis.curve));
            }
            ui.add(plot.data_aspect(1.).legend(Legend::default()));
            if self.driver.speed != 0. {
                self.driver.drive += self.driver.speed / 60.;
                ui.ctx().request_repaint();
            }
        });
    }
}
