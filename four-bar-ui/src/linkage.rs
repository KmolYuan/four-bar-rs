use crate::as_values::as_values;
use crate::csv_io::write_csv;
#[cfg(not(target_arch = "wasm32"))]
use crate::synthesis::Synthesis;
use eframe::egui::{
    plot::{Legend, Line, Plot, Points, Polygon},
    reset_button, Button, CentralPanel, Color32, CtxRef, DragValue, Ui, Widget,
};
use four_bar::{FourBar, Mechanism};
use ron::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::{
    f64::consts::{PI, TAU},
    sync::{Arc, Mutex},
};

macro_rules! unit {
    ($label:literal, $attr:expr, $inter:expr, $ui:ident) => {
        DragValue::new(&mut $attr)
            .prefix($label)
            .speed($inter)
            .ui($ui);
    };
}

macro_rules! link {
    ($label:literal, $attr:expr, $inter:expr, $ui:ident) => {
        DragValue::new(&mut $attr)
            .prefix($label)
            .clamp_range(0.0001..=9999.)
            .speed($inter)
            .ui($ui);
    };
}

macro_rules! angle {
    ($label:literal, $attr:expr, $ui:ident, $t:literal) => {
        $ui.horizontal(|ui| {
            let mut deg = $attr / PI * 180.;
            if DragValue::new(&mut deg)
                .prefix($label)
                .suffix(concat![" deg", $t])
                .clamp_range(0..=360)
                .speed(1.)
                .ui(ui)
                .changed()
            {
                $attr = deg / 180. * PI;
            }
            DragValue::new(&mut $attr)
                .suffix(concat![" rad", $t])
                .min_decimals(2)
                .clamp_range((0.)..=TAU)
                .speed(0.01)
                .ui(ui);
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
        DragValue::new(&mut $attr)
            .prefix($label)
            .clamp_range($min..=9999)
            .speed($inter)
            .ui($ui);
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
    Crank,
    Follower,
    Coupler,
}

/// Linkage data.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Linkage {
    config: Config,
    driver: Driver,
    four_bar: Arc<Mutex<FourBar>>,
    path1: Vec<[f64; 2]>,
    path2: Vec<[f64; 2]>,
    path3: Vec<[f64; 2]>,
    joints: [[f64; 2]; 5],
    pivot: Pivot,
    #[cfg(not(target_arch = "wasm32"))]
    synthesis: Synthesis,
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    save_fn: js_sys::Function,
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    load_fn: js_sys::Function,
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    load_str: js_sys::Array,
}

impl Default for Linkage {
    fn default() -> Self {
        Self {
            config: Default::default(),
            driver: Default::default(),
            four_bar: Default::default(),
            path1: Default::default(),
            path2: Default::default(),
            path3: Default::default(),
            joints: Default::default(),
            pivot: Pivot::Coupler,
            #[cfg(not(target_arch = "wasm32"))]
            synthesis: Default::default(),
            #[cfg(target_arch = "wasm32")]
            save_fn: js_sys::Function::new_no_args(""),
            #[cfg(target_arch = "wasm32")]
            load_fn: js_sys::Function::new_no_args(""),
            #[cfg(target_arch = "wasm32")]
            load_str: js_sys::Array::new(),
        }
    }
}

impl PartialEq for Linkage {
    fn eq(&self, other: &Self) -> bool {
        self.driver == other.driver
            && *self.four_bar.lock().unwrap() == *other.four_bar.lock().unwrap()
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
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn with_hook(save_fn: js_sys::Function, load_fn: js_sys::Function) -> Self {
        Self {
            save_fn,
            load_fn,
            ..Self::default()
        }
    }

    fn update_mechanism(&mut self) {
        let m = Mechanism::four_bar(self.four_bar.lock().unwrap().clone());
        m.apply(self.driver.drive, [0, 1, 2, 3, 4], &mut self.joints);
        let [path1, path2, path3] = m.four_bar_loop_all(0., self.config.curve_n);
        self.path1 = path1;
        self.path2 = path2;
        self.path3 = path3;
    }

    pub(crate) fn panel(&mut self, ui: &mut Ui) {
        self.update_mechanism();
        ui.group(|ui| {
            ui.heading("File");
            ui.horizontal(|ui| self.file_io(ui));
            ui.horizontal(|ui| self.curve_io(ui));
            ui.collapsing("Options", |ui| {
                reset_button(ui, &mut self.config);
                link!("UI value interval: ", self.config.interval, 0.01, ui);
                num!("Number of curve points: ", self.config.curve_n, 1, 10, ui);
            });
        });
        ui.group(|ui| {
            ui.heading("Dimension");
            #[cfg(not(target_arch = "wasm32"))]
            reset_button(ui, self);
            #[cfg(target_arch = "wasm32")]
            if ui
                .add_enabled(*self != Self::default(), Button::new("Reset"))
                .clicked()
            {
                *self = Self {
                    save_fn: self.save_fn.clone(),
                    load_fn: self.load_fn.clone(),
                    ..Self::default()
                }
            }
            self.parameter(ui);
        });
        ui.group(|ui| {
            ui.heading("Driver");
            reset_button(ui, &mut self.driver);
            angle!("Speed: ", self.driver.speed, ui, "/s");
            angle!("Angle: ", self.driver.drive, ui);
        });
        #[cfg(not(target_arch = "wasm32"))]
        ui.group(|ui| {
            self.synthesis.update(ui, self.four_bar.clone());
        });
    }

    fn file_io(&mut self, ui: &mut Ui) {
        #[cfg(target_arch = "wasm32")]
        if ui.button("💾 Save").clicked() {
            use js_sys::JsString;
            let this = wasm_bindgen::JsValue::NULL;
            let s = JsString::from(to_string(&*self.four_bar.lock().unwrap()).unwrap());
            let path = JsString::from("four_bar.ron");
            self.save_fn.call2(&this, &s, &path).unwrap();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("💾 Save").clicked() {
            let s = to_string(&*self.four_bar.lock().unwrap()).unwrap();
            if let Some(file_name) = rfd::FileDialog::new()
                .set_file_name("four_bar.ron")
                .add_filter("Rusty Object Notation", &["ron"])
                .save_file()
            {
                std::fs::write(file_name, s).unwrap_or_default();
            }
        }
        #[cfg(target_arch = "wasm32")]
        if ui.button("🖴 Open").clicked() {
            use js_sys::JsString;
            let this = wasm_bindgen::JsValue::NULL;
            let format = JsString::from(".ron");
            self.load_fn.call2(&this, &self.load_str, &format).unwrap();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("🖴 Open").clicked() {
            let s = if let Some(file_name) = rfd::FileDialog::new()
                .add_filter("Rusty Object Notation", &["ron"])
                .pick_file()
            {
                std::fs::read_to_string(file_name).unwrap_or_default()
            } else {
                String::new()
            };
            if let Ok(four_bar) = from_str::<FourBar>(s.as_str()) {
                *self.four_bar.lock().unwrap() = four_bar;
            }
        }
        #[cfg(target_arch = "wasm32")]
        if self.load_str.length() > 0 {
            use js_sys::JsString;
            let s = String::from(JsString::from(self.load_str.pop()));
            if let Ok(four_bar) = from_str::<FourBar>(s.as_str()) {
                *self.four_bar.lock().unwrap() = four_bar;
            }
        }
    }

    fn curve_io(&mut self, ui: &mut Ui) {
        let path = match self.pivot {
            Pivot::Crank => &self.path1,
            Pivot::Follower => &self.path2,
            Pivot::Coupler => &self.path3,
        };
        #[cfg(target_arch = "wasm32")]
        if ui.button("💾 Save Curve").clicked() {
            use js_sys::JsString;
            let this = wasm_bindgen::JsValue::NULL;
            let s = JsString::from(write_csv(path).unwrap());
            let path = JsString::from("curve.csv");
            self.save_fn.call2(&this, &s, &path).unwrap();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("💾 Save Curve").clicked() {
            let s = write_csv(path).unwrap_or_default();
            if let Some(file_name) = rfd::FileDialog::new()
                .set_file_name("curve.csv")
                .add_filter("Delimiter-Separated Values", &["txt", "csv"])
                .save_file()
            {
                std::fs::write(file_name, s).unwrap_or_default();
            }
        }
        ui.selectable_value(&mut self.pivot, Pivot::Coupler, "Coupler");
        ui.selectable_value(&mut self.pivot, Pivot::Crank, "Crank");
        ui.selectable_value(&mut self.pivot, Pivot::Follower, "Follower");
    }

    fn parameter(&mut self, ui: &mut Ui) {
        let interval = self.config.interval;
        let mut four_bar = self.four_bar.lock().unwrap();
        if ui.button("Normalize").clicked() {
            four_bar.normalize();
        }
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
            link!("Crank: ", four_bar.l1, interval, ui);
            link!("Coupler: ", four_bar.l2, interval, ui);
            link!("Follower: ", four_bar.l3, interval, ui);
            ui.checkbox(&mut four_bar.inv, "Invert follower and coupler");
        });
        ui.group(|ui| {
            ui.heading("Coupler");
            link!("Extended: ", four_bar.l4, interval, ui);
            angle!("Angle: ", four_bar.g, ui);
        });
    }

    pub(crate) fn plot(&mut self, ctx: &CtxRef) {
        CentralPanel::default().show(ctx, |ui| {
            #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
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
            #[cfg(not(target_arch = "wasm32"))]
            if !self.synthesis.curve.is_empty() {
                plot = plot.line(draw_path!("Synthesis target", self.synthesis.curve));
            }
            plot.data_aspect(1.).legend(Legend::default()).ui(ui);
            if self.driver.speed != 0. {
                self.driver.drive += self.driver.speed / 60.;
                ui.ctx().request_repaint();
            }
        });
    }
}
