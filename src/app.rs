use crate::{as_values::AsValues, linkage::*};
use eframe::{egui::*, epi};

/// Main state.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    welcome: bool,
    side_panel: bool,
    linkage: Linkage,
}

impl Default for App {
    fn default() -> Self {
        Self {
            welcome: true,
            linkage: Linkage::default(),
            side_panel: true,
        }
    }
}

impl epi::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &CtxRef, _frame: &mut epi::Frame<'_>) {
        TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ctx.style().visuals.dark_mode {
                    if ui.small_button("🔆").clicked() {
                        ctx.set_visuals(Visuals::light());
                    }
                } else {
                    if ui.small_button("🌙").clicked() {
                        ctx.set_visuals(Visuals::dark());
                    }
                }
                if self.side_panel {
                    if ui.small_button("⬅").clicked() {
                        self.side_panel = false;
                    }
                } else {
                    if ui.small_button("➡").clicked() {
                        self.side_panel = true;
                    }
                }
                ui.with_layout(Layout::right_to_left(), |ui| {
                    if ui.small_button("ℹ").clicked() {
                        self.welcome = true;
                    }
                })
            });
        });
        if self.side_panel {
            self.linkage.update(ctx);
        }
        CentralPanel::default().show(ctx, |ui| {
            let mut m = self.linkage.mechanism();
            m.four_bar_angle(0.).unwrap();
            let js = m.joints.as_values();
            let path = m.four_bar_loop(0., 360);
            plot::Plot::new("canvas")
                .line(plot::Line::new(path[0].as_values()))
                .line(plot::Line::new(path[1].as_values()))
                .line(plot::Line::new(path[2].as_values()))
                .points(plot::Points::new(js).radius(5.).color(Color32::LIGHT_BLUE))
                .data_aspect(1.)
                .ui(ui);
        });
        // Welcome message
        Window::new("Welcome to Four🍀bar!")
            .open(&mut self.welcome)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(concat!("Version: v", env!("CARGO_PKG_VERSION")));
                ui.label(env!("CARGO_PKG_DESCRIPTION"));
                ui.heading("Author");
                ui.label(env!("CARGO_PKG_AUTHORS"));
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
