use super::widgets::*;
use eframe::egui::*;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

fn pre_open(file: impl AsRef<Path>) -> Option<ColorImage> {
    if cfg!(target_arch = "wasm32") {
        None
    } else {
        std::fs::read(file)
            .ok()
            .and_then(|b| image::load_from_memory(&b).ok())
            .map(|img| ColorImage::from_rgb([img.width() as _, img.height() as _], img.as_bytes()))
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct BluePrint {
    path: Rc<RefCell<Option<PathBuf>>>,
    inner: Rc<RefCell<BpInner>>,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct BpInner {
    #[serde(skip)]
    h: Option<TextureHandle>,
    x: f64,
    y: f64,
    s: f32,
}

impl Default for BpInner {
    fn default() -> Self {
        Self { h: None, x: 0., y: 0., s: 1. }
    }
}

impl BluePrint {
    pub(crate) fn preload(&mut self, ctx: &Context) {
        let img = self.path.borrow().as_ref().and_then(pre_open);
        if let Some(img) = img {
            let h = ctx.load_texture("bp", img, Default::default());
            self.inner.borrow_mut().h.replace(h);
        } else {
            self.path.borrow_mut().take();
            *self.inner.borrow_mut() = Default::default();
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.heading("Blue Print");
        ui.label("The tool let you reference designs from pictures.");
        ui.horizontal(|ui| {
            if ui.button("🖴 Load").clicked() {
                let path = self.path.clone();
                let inner = self.inner.clone();
                let ctx = ui.ctx().clone();
                super::io::open_img(move |path_new, img| {
                    path.borrow_mut().replace(path_new);
                    let h = ctx.load_texture("bp", img, Default::default());
                    let mut inner = inner.borrow_mut();
                    *inner = Default::default();
                    inner.h.replace(h);
                });
            }
            if self.inner.borrow().h.is_some() && ui.button("🗑 Remove").clicked() {
                self.path.borrow_mut().take();
                *self.inner.borrow_mut() = Default::default();
            }
        });
        path_label(ui, "🖻", self.path.borrow().as_ref(), "No image");
        if self.inner.borrow().h.is_some() {
            let mut inner = self.inner.borrow_mut();
            unit(ui, "X coordinate: ", &mut inner.x, 1.);
            unit(ui, "Y coordinate: ", &mut inner.y, 1.);
            nonzero_f(ui, "Scale: ", &mut inner.s, 1e-2);
        }
    }

    pub(crate) fn plot(&self, ui: &mut plot::PlotUi) {
        let inner = self.inner.borrow();
        if let Some(h) = inner.h.as_ref() {
            let center = plot::PlotPoint::new(inner.x, inner.y);
            let size = h.size().map(|s| s as f32 * inner.s);
            ui.image(plot::PlotImage::new(h, center, size));
        }
    }
}
