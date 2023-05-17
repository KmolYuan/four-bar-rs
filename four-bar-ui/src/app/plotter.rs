use super::{io, widgets::*};
use eframe::egui::*;
use four_bar::*;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

pub(crate) enum Curve {
    P(Vec<[f64; 2]>),
    S(Vec<[f64; 3]>),
}

#[derive(Deserialize, Serialize)]
enum PlotType {
    P(Option<FourBar>, Vec<(String, Vec<[f64; 2]>)>),
    S(Option<SFourBar>, Vec<(String, Vec<[f64; 3]>)>),
}

impl Default for PlotType {
    fn default() -> Self {
        Self::P(None, Vec::new())
    }
}

impl PlotType {
    fn set_fb(&mut self, fb: io::Fb) {
        match (fb, self) {
            (io::Fb::Fb(fb), Self::P(p_fb, _)) => _ = p_fb.replace(fb),
            (io::Fb::Fb(fb), p @ Self::S(_, _)) => *p = Self::P(Some(fb), Vec::new()),
            (io::Fb::SFb(fb), p @ Self::P(_, _)) => *p = Self::S(Some(fb), Vec::new()),
            (io::Fb::SFb(fb), Self::S(p_fb, _)) => _ = p_fb.replace(fb),
        }
    }

    fn remove_fb(&mut self) {
        match self {
            Self::P(fb, _) => _ = fb.take(),
            Self::S(fb, _) => _ = fb.take(),
        }
    }

    fn has_fb(&self) -> bool {
        match self {
            Self::P(fb, _) => fb.is_some(),
            Self::S(fb, _) => fb.is_some(),
        }
    }

    fn push_fb_curve(&mut self, s: &'static str, c: Curve) {
        let s = s.to_string();
        match (c, self) {
            (Curve::P(c), Self::P(_, curves)) => curves.push((s, c)),
            (Curve::P(c), p @ Self::S(_, _)) => *p = Self::P(None, vec![(s, c)]),
            (Curve::S(c), p @ Self::P(_, _)) => *p = Self::S(None, vec![(s, c)]),
            (Curve::S(c), Self::S(_, curves)) => curves.push((s, c)),
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct PlotOpt {
    plot: Rc<RefCell<PlotType>>,
    angle: Option<f64>,
    inner: plot2d::OptInner,
}

impl PlotOpt {
    fn show(&mut self, ui: &mut Ui, lnk: &mut super::link::Linkages) -> bool {
        match &*self.plot.borrow() {
            PlotType::P(_, _) => ui.heading("Planar Plot"),
            PlotType::S(_, _) => ui.heading("Spherical Plot"),
        };
        if ui.button("🖴 Load Linkage").clicked() {
            let plot = self.plot.clone();
            super::io::open_ron(move |_, fb| plot.borrow_mut().set_fb(fb));
        }
        ui.horizontal(|ui| {
            if ui.button("🖴 Add from").clicked() {
                let (angle, fb) = lnk.projs.current_fb_state();
                self.plot.borrow_mut().set_fb(fb);
                self.angle.replace(angle);
            }
            lnk.projs.select(ui, false);
        });
        if self.plot.borrow().has_fb() {
            ui.horizontal(|ui| {
                let mut enable = self.angle.is_some();
                ui.checkbox(&mut enable, "Specify input angle");
                if !enable {
                    self.angle.take();
                } else if self.angle.is_none() {
                    self.angle.replace(0.);
                }
                if let Some(angle_val) = &mut self.angle {
                    angle(ui, "", angle_val, "");
                }
            });
            if ui.button("✖ Remove Linkage").clicked() {
                self.plot.borrow_mut().remove_fb();
                self.angle.take();
            }
        } else {
            ui.label("No Linkage");
        }
        ui.group(|ui| {
            ui.heading("Curves");
            match &mut *self.plot.borrow_mut() {
                PlotType::P(_, c) => c.retain_mut(|(legend, _)| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(legend);
                        !ui.button("✖").clicked()
                    })
                    .inner
                }),
                PlotType::S(_, c) => c.retain_mut(|(legend, _)| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(legend);
                        !ui.button("✖").clicked()
                    })
                    .inner
                }),
            }
            ui.horizontal(|ui| {
                if ui.button("🖴 Add from").clicked() {
                    self.plot
                        .borrow_mut()
                        .push_fb_curve("New Curve", lnk.projs.current_curve());
                }
                lnk.projs.select(ui, false);
            });
            if ui.button("🖴 Add 2D Curve from CSV").clicked() {
                let plot = self.plot.clone();
                super::io::open_csv(move |_, c| {
                    plot.borrow_mut().push_fb_curve("New Curve", Curve::P(c));
                });
            }
            if ui.button("🖴 Add 3D Curve from CSV").clicked() {
                let plot = self.plot.clone();
                super::io::open_csv(move |_, c| {
                    plot.borrow_mut().push_fb_curve("New Curve", Curve::S(c));
                });
            }
            if ui.button("🖴 Add Curve from RON").clicked() {
                let res = lnk.cfg.res;
                let plot = self.plot.clone();
                super::io::open_ron(move |_, fb| {
                    let c = match fb {
                        io::Fb::Fb(fb) => Curve::P(fb.curve(res)),
                        io::Fb::SFb(fb) => Curve::S(fb.curve(res)),
                    };
                    plot.borrow_mut().push_fb_curve("New Curve", c);
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
                .for_each(|(root, p_opt)| match &*p_opt.plot.borrow() {
                    PlotType::P(fb, c) => {
                        let curves = c.iter().map(|(s, c)| (s.as_str(), c.as_slice()));
                        let mut opt = plot2d::Opt::from(fb.as_ref()).inner(p_opt.inner.clone());
                        if let Some(angle) = p_opt.angle {
                            opt = opt.angle(angle);
                        }
                        super::io::alert(plot2d::plot(root, curves, opt), |_| ());
                    }
                    PlotType::S(fb, c) => {
                        let curves = c.iter().map(|(s, c)| (s.as_str(), c.as_slice()));
                        let mut opt = plot3d::Opt::from(fb.as_ref()).inner(p_opt.inner.clone());
                        if let Some(angle) = p_opt.angle {
                            opt = opt.angle(angle);
                        }
                        super::io::alert(plot3d::plot(root, curves, opt), |_| ());
                    }
                });
            super::io::save_svg_ask(&buf, "figure.svg");
        }
    }
}
