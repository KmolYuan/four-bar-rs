use crate::as_values::AsValues;
use eframe::egui::*;
use four_bar::Mechanism;
use std::f64::consts::{FRAC_PI_6, PI, TAU};

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
                .clamp_range((0.)..=360.)
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

macro_rules! draw_link {
    ($a:expr, $b:expr) => {
        plot::Line::new([$a, $b].as_values())
            .width(3.)
            .color(Color32::from_rgb(165, 151, 132))
    };
    ($a:expr, $b:expr $(, $c:expr)+) => {
        plot::Polygon::new([$a, $b $(, $c)+].as_values())
            .width(3.)
            .fill_alpha(0.6)
            .color(Color32::from_rgb(165, 151, 132))
    };
}

macro_rules! draw_path {
    ($name:literal, $path:expr) => {
        plot::Line::new($path.as_values()).name($name).width(3.)
    };
}

/// Linkage data.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct Linkage {
    interval: f64,
    drive: f64,
    speed: f64,
    x0: f64,
    y0: f64,
    a: f64,
    l0: f64,
    l1: f64,
    l2: f64,
    l3: f64,
    l4: f64,
    g: f64,
}

impl Default for Linkage {
    fn default() -> Self {
        Self {
            interval: 1.,
            drive: 0.,
            speed: 0.,
            x0: 0.,
            y0: 0.,
            a: 0.,
            l0: 90.,
            l1: 35.,
            l2: 70.,
            l3: 70.,
            l4: 45.,
            g: FRAC_PI_6,
        }
    }
}

impl Linkage {
    pub fn panel(&mut self, ui: &mut Ui) {
        ui.heading("Options");
        link!("Value interval: ", self.interval, 0.01, ui);
        ui.heading("Dimension");
        if ui.button("Default").clicked() {
            *self = Self::default();
        }
        if ui.button("Normalize").clicked() {
            self.x0 = 0.;
            self.y0 = 0.;
            self.a = 0.;
            self.l0 /= self.l1;
            self.l2 /= self.l1;
            self.l3 /= self.l1;
            self.l4 /= self.l1;
            self.l1 = 1.;
        }
        ui.group(|ui| {
            ui.heading("Offset");
            if ui.button("Reset").clicked() {
                self.x0 = 0.;
                self.y0 = 0.;
                self.a = 0.;
            }
            unit!("X Offset: ", self.x0, self.interval, ui);
            unit!("Y Offset: ", self.y0, self.interval, ui);
            angle!("Rotation: ", self.a, ui);
        });
        ui.group(|ui| {
            ui.heading("Parameters");
            link!("Ground: ", self.l0, self.interval, ui);
            link!("Crank: ", self.l1, self.interval, ui);
            link!("Coupler: ", self.l2, self.interval, ui);
            link!("Follower: ", self.l3, self.interval, ui);
        });
        ui.group(|ui| {
            ui.heading("Coupler");
            link!("Extended: ", self.l4, self.interval, ui);
            angle!("Angle: ", self.g, ui);
        });
        ui.group(|ui| {
            ui.heading("Driver");
            if ui.button("Reset / Stop").clicked() {
                self.speed = 0.;
                self.drive = 0.;
            }
            angle!("Speed: ", self.speed, ui, "/s");
            angle!("Angle: ", self.drive, ui);
        });
    }

    pub fn plot(&mut self, ctx: &CtxRef) {
        CentralPanel::default().show(ctx, |ui| {
            let mut m = Mechanism::four_bar(
                (self.x0, self.y0),
                self.a,
                self.l0,
                self.l1,
                self.l2,
                self.l3,
                self.l4,
                self.g,
            );
            m.four_bar_angle(self.drive).unwrap();
            let joints = m.joints.clone();
            let path = m.four_bar_loop_all(0., 360);
            plot::Plot::new("canvas")
                .line(draw_link![joints[0], joints[2]])
                .line(draw_link![joints[1], joints[3]])
                .polygon(draw_link![joints[2], joints[3], joints[4]])
                .points(
                    plot::Points::new([joints[0], joints[1]].as_values())
                        .radius(7.)
                        .color(Color32::from_rgb(93, 69, 56)),
                )
                .points(
                    plot::Points::new([joints[2], joints[3], joints[4]].as_values())
                        .radius(5.)
                        .color(Color32::from_rgb(128, 96, 77)),
                )
                .line(draw_path!("Crank pivot", path[0]))
                .line(draw_path!("Follower pivot", path[1]))
                .line(draw_path!("Coupler pivot", path[2]))
                .data_aspect(1.)
                .legend(plot::Legend::default())
                .ui(ui);
            if self.speed != 0. {
                self.drive += self.speed / 60.;
                ui.ctx().request_repaint();
            }
        });
    }
}
