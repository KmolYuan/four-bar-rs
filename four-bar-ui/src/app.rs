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

pub(crate) const APP_NAME: &str = env!("CARGO_BIN_NAME");
pub(crate) const VERSION: &str = env!("APP_VERSION");
pub(crate) const GIF_RES: usize = 60;
const LOCAL_STORAGE_TIP: &str = "\
Your last settings will be used next time.
The data will be saved in the system config or
web-browser local storage.";
const FONT: [(&str, &[u8]); 2] = [
    ("Noto", include_bytes!("../assets/GoNotoCurrent.ttf")),
    ("emoji", include_bytes!("../assets/emoji-icon-font.ttf")),
];

#[rustfmt::skip]
macro_rules! repo { () => { env!("CARGO_PKG_REPOSITORY") }; }

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

fn welcome(ui: &mut Ui, save_cfg: &mut bool) {
    ui.horizontal(|ui| {
        ui.label("Theme");
        let is_dark = ui.visuals().dark_mode;
        if ui.add(SelectableLabel::new(!is_dark, "☀ Light")).clicked() {
            ui.ctx().set_visuals(Visuals::light());
        }
        if ui.add(SelectableLabel::new(is_dark, "🌜 Dark")).clicked() {
            ui.ctx().set_visuals(Visuals::dark());
        }
    });
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(APP_NAME);
        ui.label(VERSION);
    });
    ui.label(env!("CARGO_PKG_DESCRIPTION"));
    if ui.button("📥 Download Desktop Version").clicked() {
        ui.ctx()
            .open_url(OpenUrl::new_tab(concat![repo!(), "/releases/latest"]));
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
    ui.heading("User Preferences");
    ui.horizontal(|ui| {
        let text = RichText::new("Save local data").color(Color32::GREEN);
        ui.checkbox(save_cfg, text);
        hint(ui, LOCAL_STORAGE_TIP);
    });
    ui.separator();
    let res = ui.collapsing("📚 Canvas Control Tips", |ui| {
        ui.label("Pan move: Left-drag / Drag");
        ui.label("Zoom: Ctrl+Wheel / Pinch+Stretch");
        ui.label("Box Zoom: Right-drag");
        ui.label("Reset: Double-click");
    });
    if let Some(res) = res.body_response {
        res.scroll_to_me(None);
    }
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

#[repr(transparent)]
#[derive(Deserialize, Serialize)]
struct Welcome {
    open: bool,
}

impl Default for Welcome {
    fn default() -> Self {
        Self { open: true }
    }
}

impl Welcome {
    fn invert(&mut self) {
        self.open = !self.open;
    }
}

/// Main app state.
#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct App {
    link: link::Linkages,
    syn: syn::Synthesis,
    bp: blueprint::BluePrint,
    plotter: plotter::Plotter,
    save_cfg: bool,
    #[serde(flatten)]
    welcome: Welcome,
    #[serde(skip)]
    panel: Panel,
}

impl App {
    pub(crate) fn create(files: Vec<PathBuf>) -> eframe::AppCreator<'static> {
        Box::new(|ctx| Self::new_boxed(ctx, files))
    }

    fn new_boxed(
        ctx: &eframe::CreationContext,
        files: Vec<PathBuf>,
    ) -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>> {
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
        Ok(Box::new(app))
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
            let res = ui.selectable_label(self.welcome.open, "❓");
            if res.on_hover_text("Welcome (F1)").clicked() || hotkey!(ui, F1) {
                self.welcome.invert();
            }
            url_btn(ui, "", "Repository", repo!());
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
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        Window::new("Welcome to Four🍀bar!")
            .open(&mut self.welcome.open)
            .default_size([350., 520.])
            .collapsible(false)
            .resizable(false)
            .vscroll(true)
            .show(ctx, |ui| welcome(ui, &mut self.save_cfg));
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
