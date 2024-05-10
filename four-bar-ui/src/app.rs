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

pub(crate) const GIF_RES: usize = 60;
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");
const RELEASE_URL: &str = concat![env!("CARGO_PKG_REPOSITORY"), "/releases/latest"];
const FONT: [(&str, &[u8]); 2] = [
    ("Noto", include_bytes!("../assets/GoNotoCurrent.ttf")),
    ("emoji", include_bytes!("../assets/emoji-icon-font.ttf")),
];

macro_rules! hotkey {
    ($ui:ident, $key:ident) => {
        hotkey!($ui, NONE + $key)
    };
    ($ui:ident, $mod1:ident + $key:ident) => {
        hotkey!(@$ui, Modifiers::$mod1, Key::$key)
    };
    ($ui:ident, $mod1:ident + $mod2:ident + $key:ident) => {
        hotkey!(@$ui, Modifiers::$mod1 | Modifiers::$mod2, Key::$key)
    };
    (@$ui:ident, $arg1:expr, $arg2:expr) => {
        $ui.ctx().input_mut(|s| s.consume_key($arg1, $arg2))
    };
}
pub(crate) use hotkey;

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
        for (name, font) in FONT {
            font_data.insert(name.to_string(), FontData::from_static(font));
            families.push(name.to_string());
        }
        let families = BTreeMap::from([
            (FontFamily::Proportional, families.clone()),
            (FontFamily::Monospace, families),
        ]);
        ctx.egui_ctx
            .set_fonts(FontDefinitions { font_data, families });
        ctx.egui_ctx.style_mut(|style| {
            style.override_text_style = Some(TextStyle::Body);
            const STYLE: [(TextStyle, FontId); 5] = [
                (TextStyle::Button, FontId::proportional(14.)),
                (TextStyle::Small, FontId::proportional(14.)),
                (TextStyle::Body, FontId::proportional(18.)),
                (TextStyle::Monospace, FontId::proportional(18.)),
                (TextStyle::Heading, FontId::proportional(24.)),
            ];
            for (text_style, id) in STYLE {
                style.text_styles.insert(text_style, id);
            }
        });
        let mut app = ctx
            .storage
            .and_then(|s| eframe::get_value::<Self>(s, eframe::APP_KEY))
            .unwrap_or_default();
        app.bp.preload(&ctx.egui_ctx);
        app.link.preload(files);
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

    fn menu(&mut self, ui: &mut Ui) {
        for (value, icon, text) in [
            (Panel::Linkages, "🍀", "Linkages"),
            (Panel::Synthesis, "💡", "Synthesis"),
            (Panel::Plotter, "", "Plotter"),
            (Panel::BluePrint, "🖻", "Blue Print"),
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
            if ui
                .selectable_label(!self.welcome_off, "❓")
                .on_hover_text("Welcome (F1)")
                .clicked()
                || hotkey!(ui, F1)
            {
                self.welcome_off = !self.welcome_off;
            }
            url_btn(ui, "", "Repository", REPO_URL);
        });
    }

    fn canvas(&mut self, ui: &mut Ui) {
        egui_plot::Plot::new("canvas")
            .data_aspect(1.)
            .auto_bounds([true; 2].into())
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
            Panel::Off => self.canvas(ui),
        });
    }

    fn pc_view(&mut self, ctx: &Context) {
        match self.panel {
            Panel::Linkages => side_panel(ctx, |ui| self.link.show(ui)),
            Panel::Synthesis => side_panel(ctx, |ui| self.syn.show(ui, &mut self.link)),
            Panel::Plotter => side_panel(ctx, |ui| self.plotter.show(ui, &mut self.link)),
            Panel::BluePrint => side_panel(ctx, |ui| self.bp.show(ui)),
            Panel::Off => (),
        }
        CentralPanel::default().show(ctx, |ui| self.canvas(ui));
    }

    fn welcome(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Theme");
            let mut vis = ui.visuals().clone();
            ui.selectable_value(&mut vis, Visuals::light(), "☀ Light");
            ui.selectable_value(&mut vis, Visuals::dark(), "🌜 Dark");
            ui.ctx().set_visuals(vis);
        });
        ui.separator();
        ui.label(concat!["Version: v", env!("CARGO_PKG_VERSION")]);
        ui.label(env!("CARGO_PKG_DESCRIPTION"));
        if ui.button("📥 Download Desktop Version").clicked() {
            ui.ctx().open_url(OpenUrl::new_tab(RELEASE_URL));
        }
        ui.hyperlink_to("GUI powered by egui", "https://github.com/emilk/egui/");
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
        let text = RichText::new("Save local data").color(Color32::GREEN);
        ui.checkbox(&mut self.save_cfg, text);
        ui.collapsing("📚 Control Tips", |ui| {
            ui.label("Pan move: Left-drag / Drag");
            ui.label("Zoom: Ctrl+Wheel / Pinch+Stretch");
            ui.label("Box Zoom: Right-drag");
            ui.label("Reset: Double-click");
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        {
            let mut welcome = !self.welcome_off;
            Window::new("Welcome to Four🍀bar!")
                .open(&mut welcome)
                .collapsible(false)
                .auto_sized()
                .show(ctx, |ui| self.welcome(ui));
            self.welcome_off = !welcome;
        }
        TopBottomPanel::top("menu").show(ctx, |ui| ui.horizontal(|ui| self.menu(ui)));
        if ctx.input(|s| s.screen_rect.width()) < 600. {
            self.mobile_view(ctx);
        } else {
            self.pc_view(ctx);
        }
        self.link.poll(ctx);
        crate::io::show_err_msg(frame);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.save_cfg {
            eframe::set_value(storage, eframe::APP_KEY, self);
        } else {
            storage.set_string(eframe::APP_KEY, String::new());
        }
    }
}
