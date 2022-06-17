pub use self::remote::{sha512, LoginInfo};
use self::{io_ctx::*, linkages::*, synthesis::*, widgets::*};
use eframe::egui::*;
use serde::{Deserialize, Serialize};

mod io_ctx;
mod linkages;
mod project;
mod remote;
mod synthesis;
mod widgets;

const RELEASE_URL: &str = concat![env!("CARGO_PKG_REPOSITORY"), "/releases/latest"];
const FONT: &[u8] = include_bytes!("../../assets/GoNotoCurrent.ttf");

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
    welcome_off: bool,
    panel: Panel,
    started: bool,
    ctx: IoCtx,
    linkage: Linkages,
    synthesis: Synthesis,
}

impl App {
    pub fn new(ctx: &eframe::CreationContext, files: Vec<String>) -> Self {
        let font_name = "Go-Noto".to_string();
        let mut font_def = FontDefinitions::default();
        font_def.font_data.retain(|s, _| s == "emoji-icon-font");
        font_def
            .font_data
            .insert(font_name.clone(), FontData::from_static(FONT));
        let families = vec![font_name, "emoji-icon-font".to_string()];
        font_def.families.clear();
        font_def.families.extend([
            (FontFamily::Proportional, families.clone()),
            (FontFamily::Monospace, families),
        ]);
        ctx.egui_ctx.set_fonts(font_def);
        let mut style = (*ctx.egui_ctx.style()).clone();
        for (text_style, size) in [
            (TextStyle::Small, 18.),
            (TextStyle::Body, 24.),
            (TextStyle::Monospace, 24.),
            (TextStyle::Button, 30.),
            (TextStyle::Heading, 40.),
        ] {
            let id = FontId::proportional(size);
            style.text_styles.insert(text_style, id);
        }
        ctx.egui_ctx.set_style(style);
        let mut app = ctx
            .storage
            .and_then(|s| eframe::get_value::<Self>(s, eframe::APP_KEY))
            .unwrap_or_default();
        app.linkage.open_project(files);
        app
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
                ui.allocate_space(ui.available_size());
            });
        self.welcome_off = !welcome;
    }

    fn menu(&mut self, ui: &mut Ui) {
        ui.selectable_value(&mut self.panel, Panel::Linkages, "🍀")
            .on_hover_text("Linkages");
        ui.selectable_value(&mut self.panel, Panel::Synthesis, "💡")
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
            if ui.small_button("💁").on_hover_text("Welcome").clicked() {
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

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
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
            plot::Plot::new("canvas")
                .legend(Default::default())
                .data_aspect(1.)
                .coordinates_formatter(plot::Corner::LeftBottom, Default::default())
                .show(ui, |ui| {
                    self.linkage.plot(ui);
                    self.synthesis.plot(ui);
                });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
