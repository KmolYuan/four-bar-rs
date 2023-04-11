use super::widgets::*;
use eframe::egui::*;
use four_bar::{plot2d, FourBar};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

type Curves = Vec<(String, Vec<[f64; 2]>)>;

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct PlotOpt {
    fb: Rc<RefCell<Option<FourBar>>>,
    angle: Option<f64>,
    curves: Rc<RefCell<Curves>>,
    inner: plot2d::OptInner,
}

impl PlotOpt {
    fn show(&mut self, ui: &mut Ui, lnk: &mut super::link::Linkages) -> bool {
        if ui.button("🖴 Load Linkage").clicked() {
            let opt_fb = self.fb.clone();
            super::io::open_ron(move |_, fb| {
                opt_fb.borrow_mut().replace(fb);
            });
        }
        ui.horizontal(|ui| {
            if ui.button("🖴 Add from").clicked() {
                let (fb, angle) = lnk.projs.four_bar_state();
                self.fb.borrow_mut().replace(fb);
                self.angle.replace(angle);
            }
            lnk.projs.select(ui, false);
        });
        if self.fb.borrow().is_some() {
            ui.horizontal(|ui| {
                let mut enable = self.angle.is_some();
                ui.checkbox(&mut enable, "Specify input angle");
                if !enable {
                    self.angle.take();
                } else if self.angle.is_none() {
                    self.angle = Some(0.);
                }
                if let Some(angle_val) = &mut self.angle {
                    angle(ui, "", angle_val, "");
                }
            });
            if ui.button("✖ Remove Linkage").clicked() {
                self.fb.borrow_mut().take();
                self.angle.take();
            }
        } else {
            ui.label("No Linkage");
        }
        ui.group(|ui| {
            ui.heading("Curves");
            self.curves.borrow_mut().retain_mut(|(legend, _)| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(legend);
                    !ui.button("✖").clicked()
                })
                .inner
            });
            ui.horizontal(|ui| {
                if ui.button("🖴 Add from").clicked() {
                    let c = lnk.projs.current_curve();
                    self.curves.borrow_mut().push(("New Curve".to_string(), c));
                }
                lnk.projs.select(ui, false);
            });
            if ui.button("🖴 Add Curve from CSV").clicked() {
                let curves = self.curves.clone();
                super::io::open_csv(move |_, c| {
                    curves.borrow_mut().push(("New Curve".to_string(), c));
                });
            }
            if ui.button("🖴 Add Curve from RON").clicked() {
                let res = lnk.cfg.res;
                let curves = self.curves.clone();
                super::io::open_ron(move |_, fb| {
                    let c = fb.curve(res);
                    curves.borrow_mut().push(("New Curve".to_string(), c));
                });
            }
        });
        ui.group(|ui| {
            ui.heading("Plot Option");
            nonzero_i(ui, "Stroke in plots: ", &mut self.inner.stroke, 1);
            nonzero_i(ui, "Font size in plots: ", &mut self.inner.font, 1);
            ui.checkbox(&mut self.inner.scale_bar, "Show scale bar in plots");
            ui.checkbox(&mut self.inner.grid, "Show grid in plots");
            ui.checkbox(&mut self.inner.axis, "Show axis in plots");
            ui.checkbox(&mut self.inner.dot, "Use dot curve in plots");
        });
        !ui.button("✖ Remove Subplot").clicked()
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Plotter {
    size: u32,
    shape: (usize, usize),
    queue: Vec<PlotOpt>,
}

impl Default for Plotter {
    fn default() -> Self {
        Self { size: 800, shape: (1, 1), queue: Vec::new() }
    }
}

impl Plotter {
    pub(crate) fn show(&mut self, ui: &mut Ui, lnk: &mut super::link::Linkages) {
        ui.heading("Plotter");
        nonzero_i(ui, "Plot size: ", &mut self.size, 1);
        ui.horizontal(|ui| {
            ui.label("Plot grid: (");
            nonzero_i(ui, "", &mut self.shape.0, 1);
            ui.label(", ");
            nonzero_i(ui, "", &mut self.shape.1, 1);
            ui.label(")");
        });
        ui.label(format!(
            "Capacity: {}/{}",
            self.queue.len(),
            self.shape.0 * self.shape.1
        ));
        self.queue
            .retain_mut(|opt| ui.group(|ui| opt.show(ui, lnk)).inner);
        if ui.button("⊞ Add Subplot").clicked() {
            self.queue.push(PlotOpt::default());
        }
        ui.separator();
        if ui.button("💾 Save Plot").clicked() {
            use plot2d::IntoDrawingArea as _;
            let mut buf = String::new();
            let size = (
                self.size * self.shape.0 as u32,
                self.size * self.shape.1 as u32,
            );
            let b = plot2d::SVGBackend::with_string(&mut buf, size);
            b.into_drawing_area()
                .split_evenly(self.shape)
                .into_iter()
                .zip(&self.queue)
                .for_each(|(root, p_opt)| {
                    let curves = p_opt.curves.borrow();
                    let curves = curves.iter().map(|(s, c)| (s.as_str(), c.as_slice()));
                    let fb = p_opt.fb.borrow();
                    let mut opt = plot2d::Opt::from(fb.as_ref()).inner(p_opt.inner.clone());
                    if let Some(angle) = p_opt.angle {
                        opt = opt.angle(angle);
                    }
                    super::io::alert(plot2d::plot(root, curves, opt), |_| ());
                });
            super::io::save_svg_ask(&buf, "figure.svg");
        }
    }
}
