use eframe::egui::*;
use four_bar::Mechanism;
use std::f64::consts::{FRAC_PI_6, PI, TAU};

macro_rules! unit {
    ($label:literal, $attr:expr, $ui:ident) => {
        DragValue::new(&mut $attr)
            .prefix($label)
            .speed(0.01)
            .ui($ui);
    };
}

macro_rules! link {
    ($label:literal, $attr:expr, $ui:ident) => {
        DragValue::new(&mut $attr)
            .prefix($label)
            .clamp_range(0.0001..=9999.)
            .speed(0.01)
            .ui($ui);
    };
}

macro_rules! angle {
    ($label:literal, $attr:expr, $ui:ident) => {
        $ui.horizontal(|ui| {
            let mut deg = $attr / PI * 180.;
            if DragValue::new(&mut deg)
                .prefix($label)
                .suffix(" deg")
                .clamp_range((0.)..=360.)
                .speed(0.01)
                .ui(ui)
                .changed()
            {
                $attr = deg / 180. * PI;
            }
            DragValue::new(&mut $attr)
                .suffix(" rad")
                .min_decimals(2)
                .clamp_range((0.)..=TAU)
                .speed(0.01)
                .ui(ui);
        });
    };
}

/// Linkage data.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct Linkage {
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
    pub fn mechanism(&self) -> Mechanism {
        Mechanism::four_bar(
            (self.x0, self.y0),
            self.a,
            self.l0,
            self.l1,
            self.l2,
            self.l3,
            self.l4,
            self.g,
        )
    }

    pub fn update(&mut self, ctx: &CtxRef) {
        SidePanel::left("side panel").show(ctx, |ui: &mut Ui| {
            ui.heading("Dimensional Configuration");
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.heading("Offset");
                    if ui.button("Reset").clicked() {
                        self.x0 = 0.;
                        self.y0 = 0.;
                        self.a = 0.;
                    }
                    unit!("X Offset: ", self.x0, ui);
                    unit!("Y Offset: ", self.y0, ui);
                    angle!("Rotation: ", self.a, ui);
                });
                ui.group(|ui| {
                    ui.heading("Parameters");
                    link!("Ground: ", self.l0, ui);
                    link!("Crank: ", self.l1, ui);
                    link!("Coupler: ", self.l2, ui);
                    link!("Follower: ", self.l3, ui);
                });
                ui.group(|ui| {
                    ui.heading("Coupler");
                    link!("Extended: ", self.l4, ui);
                    angle!("Angle: ", self.g, ui);
                });
            });
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                ui.add(Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"));
            });
        });
    }
}
