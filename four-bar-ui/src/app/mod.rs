pub use self::remote::{sha512, LoginInfo};
use self::{atomic::Atomic, io_ctx::IoCtx, linkage::Linkage};
use eframe::egui::{
    CtxRef, Hyperlink, Layout, ScrollArea, SidePanel, TopBottomPanel, Ui, Visuals, Window,
};
use serde::{Deserialize, Serialize};

mod atomic;
mod io_ctx;
mod linkage;
mod remote;
mod synthesis;

#[cfg(not(target_arch = "wasm32"))]
const COOKIE_KEY: &str = "cookies";

fn switch(ui: &mut Ui, attr: &mut bool, d_icon: &str, d_tip: &str, e_icon: &str, e_tip: &str) {
    if *attr {
        if ui.small_button(d_icon).on_hover_text(d_tip).clicked() {
            *attr = false;
        }
    } else if ui.small_button(e_icon).on_hover_text(e_tip).clicked() {
        *attr = true;
    }
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
            ctx: IoCtx::default(),
            linkage: Linkage::default(),
        }
    }
}

impl App {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(file: Option<&str>) -> Self {
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
        switch(
            ui,
            &mut self.menu_up,
            "⬇",
            "menu go down",
            "⬆",
            "menu go up",
        );
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

impl eframe::epi::App for App {
    fn update(&mut self, ctx: &CtxRef, _frame: &mut eframe::epi::Frame) {
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

    fn setup(
        &mut self,
        _ctx: &CtxRef,
        _frame: &mut eframe::epi::Frame<'_>,
        storage: Option<&dyn eframe::epi::Storage>,
    ) {
        if let Some(storage) = storage {
            if let Some(app) = eframe::epi::get_value(storage, eframe::epi::APP_KEY) {
                *self = app;
            }
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(cookies) = storage.get_string(COOKIE_KEY) {
                self.ctx.load_cookies(cookies);
            }
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::epi::Storage) {
        eframe::epi::set_value(storage, eframe::epi::APP_KEY, self);
        #[cfg(not(target_arch = "wasm32"))]
        storage.set_string(COOKIE_KEY, self.ctx.get_cookies());
    }

    fn name(&self) -> &str {
        "Four bar"
    }
}
