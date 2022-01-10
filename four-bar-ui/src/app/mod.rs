pub use self::remote::{sha512, LoginInfo};
use self::{io_ctx::IoCtx, linkage::Linkage, widgets::switch};
use crate::app::widgets::{switch_same, url_button};
use eframe::{
    egui::{CentralPanel, CtxRef, Layout, ScrollArea, SidePanel, TopBottomPanel, Ui, Window},
    epi::{Frame, Storage, APP_KEY},
};
use serde::{Deserialize, Serialize};

mod io_ctx;
mod linkage;
mod project;
mod remote;
mod synthesis;
mod widgets;

const RELEASE_URL: &str = concat![env!("CARGO_PKG_REPOSITORY"), "/releases/latest"];

/// Main app state.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    init_project: Vec<String>,
    welcome: bool,
    side_panel: bool,
    started: bool,
    ctx: IoCtx,
    linkage: Linkage,
}

impl Default for App {
    fn default() -> Self {
        Self {
            init_project: Vec::new(),
            welcome: true,
            side_panel: true,
            started: false,
            ctx: IoCtx::default(),
            linkage: Linkage::default(),
        }
    }
}

impl App {
    pub fn open(files: Vec<String>) -> Self {
        Self {
            init_project: files,
            ..Self::default()
        }
    }

    fn menu(&mut self, ui: &mut Ui) {
        switch(ui, "⬅", "Fold", "➡", "Expand", &mut self.side_panel);
        ui.with_layout(Layout::right_to_left(), |ui| {
            let style = ui.style().clone();
            if let Some(v) = style.visuals.light_dark_small_toggle_button(ui) {
                ui.ctx().set_visuals(v);
            }
            if ui.small_button("↻").on_hover_text("Reset UI").clicked() {
                let v = style.visuals.clone();
                *ui.ctx().memory() = Default::default();
                ui.ctx().set_visuals(v);
            }
            url_button(ui, "⮋", "Release", RELEASE_URL);
            url_button(ui, "", "Repository", env!("CARGO_PKG_REPOSITORY"));
            switch_same(ui, "ℹ", "Welcome", &mut self.welcome);
            ui.hyperlink_to("Powered by egui", "https://github.com/emilk/egui/");
        });
    }
}

impl eframe::epi::App for App {
    fn update(&mut self, ctx: &CtxRef, _frame: &Frame) {
        TopBottomPanel::top("menu").show(ctx, |ui| ui.horizontal(|ui| self.menu(ui)));
        if self.side_panel {
            SidePanel::left("side panel")
                .resizable(false)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| self.linkage.show(ui, &self.ctx));
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
        let init_project = self.init_project.clone();
        if let Some(storage) = storage {
            if let Some(app) = eframe::epi::get_value(storage, APP_KEY) {
                *self = app;
            }
        }
        for file in init_project {
            self.linkage.open_project(file);
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::epi::set_value(storage, APP_KEY, self);
    }

    fn name(&self) -> &str {
        "Four bar"
    }
}
