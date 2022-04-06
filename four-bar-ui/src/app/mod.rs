pub use self::remote::{sha512, LoginInfo};
use self::{io_ctx::IoCtx, linkages::Linkages, synthesis::Synthesis};
use crate::app::widgets::url_button;
use eframe::{
    egui::{
        plot::{Legend, Plot},
        CentralPanel, Context, Layout, ScrollArea, SidePanel, TopBottomPanel, Ui, Window,
    },
    epi::{Frame, Storage, APP_KEY},
};
use serde::{Deserialize, Serialize};

mod io_ctx;
mod linkages;
mod project;
mod remote;
mod synthesis;
mod widgets;

const RELEASE_URL: &str = concat![env!("CARGO_PKG_REPOSITORY"), "/releases/latest"];

#[derive(Deserialize, Serialize, PartialEq)]
enum Panel {
    Linkages,
    Synthesis,
    Monitor,
    Off,
}

impl Default for Panel {
    fn default() -> Self {
        Self::Linkages
    }
}

/// Main app state.
#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    init_project: Vec<String>,
    welcome_off: bool,
    panel: Panel,
    started: bool,
    ctx: IoCtx,
    linkage: Linkages,
    synthesis: Synthesis,
}

impl App {
    pub fn open(files: Vec<String>) -> Self {
        Self {
            init_project: files,
            ..Self::default()
        }
    }

    fn welcome(&mut self, ctx: &Context) {
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
        ui.selectable_value(&mut self.panel, Panel::Linkages, "🍀")
            .on_hover_text("Linkages");
        ui.selectable_value(&mut self.panel, Panel::Synthesis, "⛓")
            .on_hover_text("Synthesis");
        ui.selectable_value(&mut self.panel, Panel::Monitor, "🖥")
            .on_hover_text("Renderer Monitor");
        ui.selectable_value(&mut self.panel, Panel::Off, "⛶")
            .on_hover_text("Close Panel");
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
            if ui.small_button("ℹ").on_hover_text("Welcome").clicked() {
                self.welcome_off = !self.welcome_off;
            }
            ui.hyperlink_to("Powered by egui", "https://github.com/emilk/egui/");
        });
    }

    fn side_panel(ctx: &Context, f: impl FnOnce(&mut Ui)) {
        SidePanel::left("side panel")
            .resizable(false)
            .show(ctx, |ui| ScrollArea::vertical().show(ui, f));
    }
}

impl eframe::epi::App for App {
    fn update(&mut self, ctx: &Context, _frame: &Frame) {
        self.welcome(ctx);
        TopBottomPanel::top("menu").show(ctx, |ui| ui.horizontal(|ui| self.menu(ui)));
        match self.panel {
            Panel::Linkages => Self::side_panel(ctx, |ui| self.linkage.show(ui)),
            Panel::Synthesis => Self::side_panel(ctx, |ui| {
                self.synthesis.show(ui, &self.ctx, &mut self.linkage)
            }),
            Panel::Monitor => Self::side_panel(ctx, |ui| {
                ui.heading("Renderer Monitor");
                ctx.memory_ui(ui);
                ctx.inspection_ui(ui);
            }),
            Panel::Off => (),
        }
        CentralPanel::default().show(ctx, |ui| {
            Plot::new("canvas")
                .data_aspect(1.)
                .legend(Legend::default())
                .show(ui, |ui| {
                    self.linkage.plot(ui);
                    self.synthesis.plot(ui);
                });
        });
    }

    fn setup(&mut self, _ctx: &Context, _frame: &Frame, storage: Option<&dyn Storage>) {
        let init_proj = self.init_project.clone();
        if let Some(storage) = storage {
            if let Some(app) = eframe::epi::get_value(storage, APP_KEY) {
                *self = app;
            }
        }
        self.linkage.open_project(init_proj);
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::epi::set_value(storage, APP_KEY, self);
    }

    fn name(&self) -> &str {
        "Four bar"
    }
}
