use crate::linkage::*;
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
                    if ui.small_button("🔆").on_hover_text("Light").clicked() {
                        ctx.set_visuals(Visuals::light());
                    }
                } else {
                    if ui.small_button("🌙").on_hover_text("Dark").clicked() {
                        ctx.set_visuals(Visuals::dark());
                    }
                }
                if self.side_panel {
                    if ui.small_button("⬅").on_hover_text("Fold").clicked() {
                        self.side_panel = false;
                    }
                } else {
                    if ui.small_button("➡").on_hover_text("Expand").clicked() {
                        self.side_panel = true;
                    }
                }
                ui.with_layout(Layout::right_to_left(), |ui| {
                    if ui.small_button("ℹ").on_hover_text("Welcome").clicked() {
                        self.welcome = true;
                    }
                    if ui
                        .small_button("↻")
                        .on_hover_text("Reset UI Setting")
                        .clicked()
                    {
                        *ctx.memory() = Default::default();
                    }
                });
            });
        });
        if self.side_panel {
            self.linkage.update(ctx);
        }
        self.linkage.plot(ctx);
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
