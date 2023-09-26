use super::widgets::*;
use crate::io;
use eframe::egui::*;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

fn pre_open(file: impl AsRef<std::path::Path>) -> Option<ColorImage> {
    if cfg!(target_arch = "wasm32") {
        None
    } else {
        io::load_img(std::fs::File::open(file).ok()?).ok()
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct BluePrint {
    path: io::Cache<std::path::PathBuf>,
    info: Rc<RefCell<BpInfo>>,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct BpInfo {
    #[serde(skip)]
    h: Option<TextureHandle>,
    x: f64,
    y: f64,
    s: f32,
}

impl Default for BpInfo {
    fn default() -> Self {
        Self { h: None, x: 0., y: 0., s: 1. }
    }
}

impl BluePrint {
    pub(crate) fn preload(&mut self, ctx: &Context) {
        let Some(path) = &*self.path.borrow() else {
            return;
        };
        if let Some(img) = pre_open(path) {
            let h = ctx.load_texture("bp", img, Default::default());
            self.info.borrow_mut().h.replace(h);
        } else {
            self.path.borrow_mut().take();
            *self.info.borrow_mut() = Default::default();
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.heading("Blue Print");
        ui.label("The tool let you reference designs from pictures.");
        ui.horizontal(|ui| {
            if ui.button("🖴 Load").clicked() {
                let path = self.path.clone();
                let inner = self.info.clone();
                let ctx = ui.ctx().clone();
                io::open_img(move |path_new, img| {
                    path.borrow_mut().replace(path_new);
                    let h = ctx.load_texture("bp", img, Default::default());
                    let mut inner = inner.borrow_mut();
                    *inner = Default::default();
                    inner.h.replace(h);
                });
            }
            if self.info.borrow().h.is_some() && ui.button("✖ Remove").clicked() {
                self.path.borrow_mut().take();
                *self.info.borrow_mut() = Default::default();
            }
        });
        path_label(ui, "🖻", self.path.borrow().as_ref(), "No image");
        if self.info.borrow().h.is_some() {
            let mut info = self.info.borrow_mut();
            unit(ui, "X coordinate: ", &mut info.x, 1e-2);
            unit(ui, "Y coordinate: ", &mut info.y, 1e-2);
            nonzero_f(ui, "Scale: ", &mut info.s, 1e-4);
        }
    }

    pub(crate) fn plot(&self, ui: &mut plot::PlotUi) {
        let inner = self.info.borrow();
        if let Some(h) = inner.h.as_ref() {
            let center = plot::PlotPoint::new(inner.x, inner.y);
            let size = h.size().map(|s| s as f32 * inner.s);
            ui.image(plot::PlotImage::new(h, center, size));
        }
    }
}
