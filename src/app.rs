use eframe::{egui::*, epi};
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

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    welcome: bool,
    x0: f64,
    y0: f64,
    alpha: f64,
    l0: f64,
    l1: f64,
    l2: f64,
    l3: f64,
    l4: f64,
    gamma: f64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            welcome: true,
            x0: 0.,
            y0: 0.,
            alpha: 0.,
            l0: 90.,
            l1: 35.,
            l2: 70.,
            l3: 70.,
            l4: 45.,
            gamma: FRAC_PI_6,
        }
    }
}

impl epi::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &CtxRef, _frame: &mut epi::Frame<'_>) {
        SidePanel::left("side panel").show(ctx, |ui| {
            ui.heading("Dimensional Configuration");
            ui.group(|ui| {
                ui.heading("Offset");
                if ui.button("Reset").clicked() {
                    self.x0 = 0.;
                    self.y0 = 0.;
                    self.alpha = 0.;
                }
                unit!("X Offset: ", self.x0, ui);
                unit!("Y Offset: ", self.y0, ui);
                angle!("Rotation: ", self.alpha, ui);
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
                angle!("Angle: ", self.gamma, ui);
            });
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                ui.add(Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"));
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            plot::Plot::new("canvas").ui(ui);
        });
        // Welcome message
        Window::new("Welcome to Four🍀bar!")
            .open(&mut self.welcome)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(
                    "Four-bar is a four-bar linkage mechanism \
                    simulator and synthesizing tool.",
                );
                ui.heading("License");
                ui.label("This software is under AGPL v3 license.");
                ui.label("The commercial usages under server or client side are not allowed.");
            });
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn name(&self) -> &str {
        "Four🍀bar"
    }
}
