use self::widgets::*;
use eframe::egui::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

mod blueprint;
mod link;
mod plotter;
mod proj;
mod syn;
mod widgets;

const RELEASE_URL: &str = concat![env!("CARGO_PKG_REPOSITORY"), "/releases/latest"];
const FONT: &[(&str, &[u8])] = &[
    ("Noto", include_bytes!("../assets/GoNotoCurrent.ttf")),
    ("emoji", include_bytes!("../assets/emoji-icon-font.ttf")),
];

fn side_panel(ctx: &Context, f: impl FnOnce(&mut Ui)) {
    SidePanel::left("side").show(ctx, |ui| ScrollArea::vertical().show(ui, f));
}

fn pan_panel(ui: &mut Ui, f: impl FnOnce(&mut Ui)) {
    ScrollArea::vertical().show(ui, f);
}

#[derive(Default, PartialEq, Eq)]
enum Panel {
    #[default]
    Linkages,
    Synthesis,
    Plotter,
    BluePrint,
    Options,
    Off,
}

/// Main app state.
#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct App {
    welcome_off: bool,
    link: link::Linkages,
    syn: syn::Synthesis,
    bp: blueprint::BluePrint,
    plotter: plotter::Plotter,
    save_cfg: bool,
    #[serde(skip)]
    panel: Panel,
}

impl App {
    pub(crate) fn create(files: Vec<PathBuf>) -> eframe::AppCreator {
        Box::new(|ctx| Self::new_boxed(ctx, files))
    }

    fn new_boxed(ctx: &eframe::CreationContext, files: Vec<PathBuf>) -> Box<Self> {
        let mut font_data = BTreeMap::new();
        let mut families = Vec::with_capacity(FONT.len());
        for &(name, font) in FONT {
            font_data.insert(name.to_string(), FontData::from_static(font));
            families.push(name.to_string());
        }
        let families = BTreeMap::from([
            (FontFamily::Proportional, families.clone()),
            (FontFamily::Monospace, families),
        ]);
        ctx.egui_ctx
            .set_fonts(FontDefinitions { font_data, families });
        let mut style = (*ctx.egui_ctx.style()).clone();
        style.override_text_style = Some(TextStyle::Body);
        for (text_style, size) in [
            (TextStyle::Button, 14.),
            (TextStyle::Small, 14.),
            (TextStyle::Body, 18.),
            (TextStyle::Monospace, 18.),
            (TextStyle::Heading, 24.),
        ] {
            let id = FontId::proportional(size);
            style.text_styles.insert(text_style, id);
        }
        ctx.egui_ctx.set_style(style);
        let mut app = ctx
            .storage
            .and_then(|s| eframe::get_value::<Self>(s, eframe::APP_KEY))
            .unwrap_or_default();
        app.bp.preload(&ctx.egui_ctx);
        app.link.preload(files, app.link.cfg.res);
        #[cfg(target_arch = "wasm32")]
        {
            #[wasm_bindgen::prelude::wasm_bindgen]
            extern "C" {
                fn load_url() -> String;
                fn loading_finished();
            }
            if let Ok(fb) = ron::from_str(&load_url()) {
                app.link.projs.queue().push(None, fb);
            }
            loading_finished();
        }
        Box::new(app)
    }

    fn welcome(&mut self, ctx: &Context) {
        let mut welcome = !self.welcome_off;
        Window::new("Welcome to Four🍀bar!")
            .open(&mut welcome)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(concat!["Version: v", env!("CARGO_PKG_VERSION")]);
                ui.label(env!("CARGO_PKG_DESCRIPTION"));
                ui.horizontal(|ui| {
                    url_btn(ui, "", "Repository", env!("CARGO_PKG_REPOSITORY"));
                    url_btn(ui, "⮋", "Release", RELEASE_URL);
                });
                ui.hyperlink_to("Powered by egui", "https://github.com/emilk/egui/");
                ui.separator();
                ui.heading("Author");
                ui.label(env!("CARGO_PKG_AUTHORS"));
                ui.separator();
                ui.heading("License");
                ui.label("This software is under AGPL v3 license.");
                ui.label("The commercial usages under server or client side are not allowed.");
                ui.separator();
                ui.heading("Local Storage");
                ui.label("The local storage is disabled by default.");
                let text = WidgetText::from("Save local data").color(Color32::GREEN);
                ui.checkbox(&mut self.save_cfg, text);
                ui.allocate_space(ui.available_size());
            });
        self.welcome_off = !welcome;
    }

    fn menu(&mut self, ui: &mut Ui) {
        for (value, icon, text) in [
            (Panel::Linkages, "🍀", "Linkages"),
            (Panel::Synthesis, "💡", "Synthesis"),
            (Panel::Plotter, "", "Plotter"),
            (Panel::BluePrint, "🖻", "Blue Print"),
            (Panel::Options, "🛠", "Options"),
        ] {
            let is_current = self.panel == value;
            if ui
                .selectable_label(is_current, icon)
                .on_hover_text(text)
                .clicked()
            {
                if is_current {
                    self.panel = Panel::Off;
                } else {
                    self.panel = value;
                }
            }
        }
        ui.with_layout(Layout::right_to_left(Align::LEFT), |ui| {
            if small_btn(ui, "❓", "Welcome") {
                self.welcome_off = !self.welcome_off;
            }
        });
    }

    fn canvas(&mut self, ui: &mut Ui) {
        egui_plot::Plot::new("canvas")
            .data_aspect(1.)
            .auto_bounds_x()
            .auto_bounds_y()
            .legend(Default::default())
            .coordinates_formatter(egui_plot::Corner::LeftBottom, Default::default())
            .show(ui, |ui| {
                self.bp.plot(ui);
                self.link.plot(ui);
                self.syn.plot(ui, &self.link);
            });
    }

    fn mobile_view(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| match self.panel {
            Panel::Linkages => pan_panel(ui, |ui| self.link.show(ui)),
            Panel::Synthesis => pan_panel(ui, |ui| self.syn.show(ui, &mut self.link)),
            Panel::Plotter => pan_panel(ui, |ui| self.plotter.show(ui, &mut self.link)),
            Panel::BluePrint => pan_panel(ui, |ui| self.bp.show(ui)),
            Panel::Options => pan_panel(ui, |ui| self.link.option(ui)),
            Panel::Off => self.canvas(ui),
        });
    }

    fn pc_view(&mut self, ctx: &Context) {
        match self.panel {
            Panel::Linkages => side_panel(ctx, |ui| self.link.show(ui)),
            Panel::Synthesis => side_panel(ctx, |ui| self.syn.show(ui, &mut self.link)),
            Panel::Plotter => side_panel(ctx, |ui| self.plotter.show(ui, &mut self.link)),
            Panel::BluePrint => side_panel(ctx, |ui| self.bp.show(ui)),
            Panel::Options => side_panel(ctx, |ui| self.link.option(ui)),
            Panel::Off => (),
        }
        CentralPanel::default().show(ctx, |ui| self.canvas(ui));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        self.welcome(ctx);
        TopBottomPanel::top("menu").show(ctx, |ui| ui.horizontal(|ui| self.menu(ui)));
        if ctx.input(|s| s.screen_rect.width()) < 600. {
            self.mobile_view(ctx);
        } else {
            self.pc_view(ctx);
        }
        self.link.poll(ctx);
        crate::io::push_err_msg(frame);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.save_cfg {
            eframe::set_value(storage, eframe::APP_KEY, self);
        } else {
            storage.set_string(eframe::APP_KEY, String::new());
        }
    }
}
