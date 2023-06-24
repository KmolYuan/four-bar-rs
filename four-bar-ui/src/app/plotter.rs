use super::widgets::*;
use crate::io;
use eframe::egui::*;
use four_bar::*;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

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

    fn push_fb_curve(&mut self, s: &'static str, c: io::Curve) {
        let s = s.to_string();
        match (c, self) {
            (io::Curve::P(c), Self::P(_, curves)) => curves.push((s, c)),
            (io::Curve::P(c), p @ Self::S(_, _)) => *p = Self::P(None, vec![(s, c)]),
            (io::Curve::S(c), p @ Self::P(_, _)) => *p = Self::S(None, vec![(s, c)]),
            (io::Curve::S(c), Self::S(_, curves)) => curves.push((s, c)),
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct PlotOpt {
    plot: Rc<RefCell<PlotType>>,
    angle: Option<f64>,
    opt: plot2d::Opt<'static>,
}

impl PlotOpt {
    fn show(&mut self, ui: &mut Ui, lnk: &mut super::link::Linkages) -> bool {
        match &*self.plot.borrow() {
            PlotType::P(_, _) => ui.heading("Planar Plot"),
            PlotType::S(_, _) => ui.heading("Spherical Plot"),
        };
        if ui.button("🖴 Load Linkage").clicked() {
            let plot = self.plot.clone();
            io::open_ron(move |_, fb| plot.borrow_mut().set_fb(fb));
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
            check_on(ui, "Input angle", &mut self.angle, angle_f);
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
            if ui.button("🖴 Add Curve from CSV").clicked() {
                let plot = self.plot.clone();
                io::open_csv(move |_, c| plot.borrow_mut().push_fb_curve("New Curve", c));
            }
            if ui.button("🖴 Add Curve from RON").clicked() {
                let res = lnk.cfg.res;
                let plot = self.plot.clone();
                io::open_ron(move |_, fb| {
                    plot.borrow_mut()
                        .push_fb_curve("New Curve", fb.into_curve(res));
                });
            }
        });
        ui.group(|ui| {
            ui.heading("Plot Option");
            check_on(ui, "Title", &mut self.opt.title, |ui, s| {
                ui.text_edit_singleline(s.to_mut())
            });
            nonzero_i(ui, "Stroke in plots: ", &mut self.opt.stroke, 1);
            nonzero_i(ui, "Font size in plots: ", &mut self.opt.font, 1);
            check_on(ui, "Font Family", &mut self.opt.font_family, |ui, s| {
                ui.text_edit_singleline(s.to_mut())
            });
            ui.checkbox(&mut self.opt.grid, "Show grid in plots");
            ui.checkbox(&mut self.opt.axis, "Show axis in plots");
            ComboBox::new("legend", "Legend Position")
                .selected_text(self.opt.legend.name())
                .show_ui(ui, |ui| {
                    use plot2d::LegendPos::*;
                    for pos in [Hide, UL, ML, LL, UM, MM, LM, UR, MR, LR] {
                        ui.selectable_value(&mut self.opt.legend, pos, pos.name());
                    }
                });
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
                        let mut fig = plot2d::Figure::from(fb.as_ref()).with_opt(p_opt.opt.clone());
                        if let Some(angle) = p_opt.angle {
                            fig = fig.angle(angle);
                        }
                        for (s, c) in c {
                            fig = fig.add_line(s, c, plot2d::Style::Circle, plot2d::BLACK);
                        }
                        io::alert(fig.plot(root), |_| ());
                    }
                    PlotType::S(fb, c) => {
                        let mut fig = plot3d::Figure::from(fb.as_ref()).with_opt(p_opt.opt.clone());
                        if let Some(angle) = p_opt.angle {
                            fig = fig.angle(angle);
                        }
                        for (s, c) in c {
                            fig = fig.add_line(s, c, plot3d::Style::Circle, plot3d::BLACK);
                        }
                        io::alert(fig.plot(root), |_| ());
                    }
                });
            io::save_svg_ask(&buf, "figure.svg");
        }
    }
}
