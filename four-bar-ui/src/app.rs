use crate::linkage::Linkage;
use eframe::{egui::*, epi};

#[macro_export]
macro_rules! switch_button {
    ($ui:expr, $attr:expr, $d_icon:literal, $d_tip:literal, $e_icon:literal, $e_tip:literal) => {
        if $attr {
            if $ui.small_button($d_icon).on_hover_text($d_tip).clicked() {
                $attr = false;
            }
        } else {
            if $ui.small_button($e_icon).on_hover_text($e_tip).clicked() {
                $attr = true;
            }
        }
    };
}

/// Main state.
#[cfg_attr(
    feature = "persistence",
    derive(serde::Deserialize, serde::Serialize),
    serde(default)
)]
pub struct App {
    welcome: bool,
    menu_up: bool,
    side_panel: bool,
    started: bool,
    linkage: Linkage,
}

impl Default for App {
    fn default() -> Self {
        Self {
            welcome: true,
            menu_up: true,
            side_panel: true,
            started: false,
            linkage: Linkage::default(),
        }
    }
}

impl App {
    fn menu(&mut self, ctx: &CtxRef, ui: &mut Ui) {
        if ctx.style().visuals.dark_mode {
            if ui.small_button("🔆").on_hover_text("Light").clicked() {
                ctx.set_visuals(Visuals::light());
            }
        } else if ui.small_button("🌙").on_hover_text("Dark").clicked() {
            ctx.set_visuals(Visuals::dark());
        }
        switch_button!(ui, self.side_panel, "⬅", "Fold", "➡", "Expand");
        switch_button!(ui, self.menu_up, "⬇", "Menu go down", "⬆", "Menu go up");
        ui.with_layout(Layout::right_to_left(), |ui| {
            if ui.small_button("ℹ").on_hover_text("Welcome").clicked() {
                self.welcome = !self.welcome;
            }
            if ui
                .small_button("↻")
                .on_hover_text("Reset UI setting")
                .clicked()
            {
                let dark = ctx.style().visuals.dark_mode;
                *ctx.memory() = Default::default();
                if dark {
                    ctx.set_visuals(Visuals::dark());
                } else {
                    ctx.set_visuals(Visuals::light());
                }
            }
        });
    }

    fn credit(ui: &mut Ui) {
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.add(Hyperlink::new("https://github.com/emilk/egui/").text("Powered by egui"));
        });
    }
}

impl epi::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &CtxRef, _frame: &mut epi::Frame<'_>) {
        if self.menu_up {
            TopBottomPanel::top("menu")
        } else {
            TopBottomPanel::bottom("menu")
        }
        .show(ctx, |ui| ui.horizontal(|ui| self.menu(ctx, ui)));
        if self.side_panel {
            SidePanel::left("side panel").show(ctx, |ui| {
                self.linkage.panel(ui);
                Self::credit(ui);
            });
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
