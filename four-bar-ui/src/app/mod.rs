pub use self::remote::{sha512, LoginInfo};
use self::{atomic::Atomic, io_ctx::IoCtx, linkage::Linkage};
use eframe::{
    egui::{CtxRef, Hyperlink, Layout, ScrollArea, SidePanel, TopBottomPanel, Ui, Visuals, Window},
    epi,
};
use serde::{Deserialize, Serialize};

mod atomic;
mod io_ctx;
mod linkage;
mod remote;
mod synthesis;

macro_rules! switch {
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

/// Main app state.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    welcome: bool,
    menu_up: bool,
    side_panel: bool,
    started: bool,
    #[serde(skip)]
    ctx: IoCtx,
    linkage: Linkage,
}

impl Default for App {
    fn default() -> Self {
        Self {
            welcome: true,
            menu_up: true,
            side_panel: true,
            started: false,
            ctx: Default::default(),
            linkage: Default::default(),
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
        switch!(ui, self.side_panel, "⬅", "Fold", "➡", "Expand");
        switch!(ui, self.menu_up, "⬇", "menu go down", "⬆", "menu go up");
        ui.with_layout(Layout::right_to_left(), |ui| {
            if ui.small_button("").on_hover_text("Repository").clicked() {
                ctx.output().open_url(env!("CARGO_PKG_REPOSITORY"));
            }
            if ui.small_button("⮋").on_hover_text("Release").clicked() {
                ctx.output()
                    .open_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/releases/latest"));
            }
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
            ui.add(Hyperlink::new("https://github.com/emilk/egui/").text("Powered by egui"));
        });
    }
}

impl epi::App for App {
    fn update(&mut self, ctx: &CtxRef, _frame: &mut epi::Frame) {
        if self.menu_up {
            TopBottomPanel::top("menu")
        } else {
            TopBottomPanel::bottom("menu")
        }
        .show(ctx, |ui| ui.horizontal(|ui| self.menu(ctx, ui)));
        if self.side_panel {
            SidePanel::left("side panel")
                .resizable(false)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| self.linkage.ui(ui, &self.ctx));
                });
        }
        self.linkage.plot(ctx);
        // Welcome message (shown in central area)
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

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn name(&self) -> &str {
        "Four bar"
    }
}
