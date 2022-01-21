pub use self::remote::{sha512, LoginInfo};
use self::{io_ctx::IoCtx, linkage::Linkage};
use crate::app::widgets::{switch_same, url_button};
use eframe::{
    egui::{
        plot::{Legend, Plot},
        CentralPanel, CtxRef, Layout, ScrollArea, SidePanel, TopBottomPanel, Ui, Window,
    },
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

#[derive(Deserialize, Serialize, PartialEq)]
enum PanelState {
    On,
    Monitor,
    Off,
}

impl Default for PanelState {
    fn default() -> Self {
        Self::On
    }
}

/// Main app state.
#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    init_project: Vec<String>,
    welcome_off: bool,
    panel: PanelState,
    started: bool,
    ctx: IoCtx,
    linkage: Linkage,
}

impl App {
    pub fn open(files: Vec<String>) -> Self {
        Self {
            init_project: files,
            ..Self::default()
        }
    }

    fn welcome(&mut self, ctx: &CtxRef) {
        let mut welcome = !self.welcome_off;
        Window::new("Welcome to Four🍀bar!")
            .open(&mut welcome)
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
        self.welcome_off = !welcome;
    }

    fn menu(&mut self, ui: &mut Ui) {
        ui.selectable_value(&mut self.panel, PanelState::On, "🍀");
        ui.selectable_value(&mut self.panel, PanelState::Monitor, "🖥");
        ui.selectable_value(&mut self.panel, PanelState::Off, "⛶");
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
            switch_same(ui, "ℹ", "Welcome", &mut self.welcome_off);
            ui.hyperlink_to("Powered by egui", "https://github.com/emilk/egui/");
        });
    }
}

impl eframe::epi::App for App {
    fn update(&mut self, ctx: &CtxRef, _frame: &Frame) {
        self.welcome(ctx);
        TopBottomPanel::top("menu").show(ctx, |ui| ui.horizontal(|ui| self.menu(ui)));
        if let PanelState::On | PanelState::Monitor = self.panel {
            SidePanel::left("side panel")
                .resizable(false)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| match self.panel {
                        PanelState::On => self.linkage.show(ui, &self.ctx),
                        PanelState::Monitor => {
                            ctx.memory_ui(ui);
                            ctx.inspection_ui(ui);
                        }
                        PanelState::Off => unreachable!(),
                    })
                });
        }
        CentralPanel::default().show(ctx, |ui| {
            Plot::new("canvas")
                .data_aspect(1.)
                .legend(Legend::default())
                .show(ui, |ui| self.linkage.plot(ui));
        });
    }

    fn setup(&mut self, _ctx: &CtxRef, _frame: &Frame, storage: Option<&dyn Storage>) {
        let mut init_proj = self.init_project.clone();
        if let Some(storage) = storage {
            if let Some(app) = eframe::epi::get_value(storage, APP_KEY) {
                *self = app;
                init_proj.append(&mut self.linkage.reload_projects());
            }
        }
        for file in init_proj {
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
