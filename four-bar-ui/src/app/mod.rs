pub use self::remote::{sha512, LoginInfo};
use self::{io_ctx::IoCtx, linkage::Linkage, widgets::switch};
use eframe::{
    egui::{
        CentralPanel, CtxRef, Hyperlink, Layout, ScrollArea, SidePanel, TopBottomPanel, Ui,
        Visuals, Window,
    },
    epi::{Frame, Storage, APP_KEY},
};
use serde::{Deserialize, Serialize};

mod canvas;
mod io_ctx;
mod linkage;
mod remote;
mod synthesis;
mod widgets;

/// Main app state.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    welcome: bool,
    menu_up: bool,
    side_panel: bool,
    started: bool,
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
            ctx: IoCtx::default(),
            linkage: Linkage::default(),
        }
    }
}

impl App {
    pub fn open(file: Option<String>) -> Self {
        Self {
            linkage: Linkage::open(file),
            ..Self::default()
        }
    }

    fn menu(&mut self, ctx: &CtxRef, ui: &mut Ui) {
        if ctx.style().visuals.dark_mode {
            if ui.small_button("🔆").on_hover_text("Light").clicked() {
                ctx.set_visuals(Visuals::light());
            }
        } else if ui.small_button("🌙").on_hover_text("Dark").clicked() {
            ctx.set_visuals(Visuals::dark());
        }
        switch(ui, &mut self.side_panel, "⬅", "Fold", "➡", "Expand");
        switch(ui, &mut self.menu_up, "⬇", "menu down", "⬆", "menu up");
        ui.with_layout(Layout::right_to_left(), |ui| {
            if ui.small_button("").on_hover_text("Repository").clicked() {
                ctx.output().open_url(env!("CARGO_PKG_REPOSITORY"));
            }
            if ui.small_button("⮋").on_hover_text("Release").clicked() {
                ctx.output()
                    .open_url(concat![env!("CARGO_PKG_REPOSITORY"), "/releases/latest"]);
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
            ui.add(Hyperlink::from_label_and_url(
                "Powered by egui",
                "https://github.com/emilk/egui/",
            ));
        });
    }
}

impl eframe::epi::App for App {
    fn update(&mut self, ctx: &CtxRef, _frame: &Frame) {
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
        CentralPanel::default().show(ctx, |ui| self.linkage.plot(ui));
        // Welcome message (shown in central area)
        Window::new("Welcome to Four🍀bar!")
            .open(&mut self.welcome)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(concat!["Version: v", env!("CARGO_PKG_VERSION")]);
                ui.label(env!("CARGO_PKG_DESCRIPTION"));
                ui.heading("Author");
                ui.label(env!("CARGO_PKG_AUTHORS"));
                ui.heading("License");
                ui.label("This software is under AGPL v3 license.");
                ui.label("The commercial usages under server or client side are not allowed.");
            });
    }

    fn setup(&mut self, _ctx: &CtxRef, _frame: &Frame, storage: Option<&dyn Storage>) {
        if let Some(storage) = storage {
            if let Some(app) = eframe::epi::get_value(storage, APP_KEY) {
                *self = app;
            }
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::epi::set_value(storage, APP_KEY, self);
    }

    fn name(&self) -> &str {
        "Four bar"
    }
}
