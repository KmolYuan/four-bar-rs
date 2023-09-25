use super::widgets::*;
use crate::io::{self, Alert as _};
use eframe::egui::*;
use four_bar::{plot as fb_plot, *};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

#[derive(Deserialize, Serialize, Default)]
struct LineData<const N: usize> {
    label: String,
    #[serde(bound(
        serialize = "[f64; N]: Serialize",
        deserialize = "[f64; N]: serde::de::DeserializeOwned"
    ))]
    line: Vec<[f64; N]>,
    style: fb_plot::Style,
    color: [u8; 3],
    stroke_width: u32,
    filled: bool,
}

impl<const N: usize> LineData<N> {
    fn new(label: String, line: Vec<[f64; N]>) -> Self {
        Self { label, line, ..Self::default() }
    }

    fn show(&mut self, ui: &mut Ui, id: Id) -> bool {
        // Line style settings
        let keep = ui
            .horizontal(|ui| {
                ui.text_edit_singleline(&mut self.label);
                !ui.button("✖").clicked()
            })
            .inner;
        ui.horizontal(|ui| {
            ui.label("Style");
            combo_enum(ui, id, &mut self.style, fb_plot::Style::LIST, |e| e.name());
            nonzero_i(ui, "Stroke Width: ", &mut self.stroke_width, 1);
        });
        ui.horizontal(|ui| {
            ui.color_edit_button_srgb(&mut self.color);
            any_i(ui, &mut self.color[0]);
            any_i(ui, &mut self.color[1]);
            any_i(ui, &mut self.color[2]);
            ui.checkbox(&mut self.filled, "Filled");
        });
        keep
    }

    fn share(&self) -> (&String, &Vec<[f64; N]>, fb_plot::Style, fb_plot::ShapeStyle) {
        let Self { style, color: [r, g, b], stroke_width, filled, .. } = *self;
        let color = {
            let color = fb_plot::RGBAColor(r, g, b, 1.);
            fb_plot::ShapeStyle { color, filled, stroke_width }
        };
        let Self { label, line, .. } = self;
        (label, line, style, color)
    }
}

#[derive(Deserialize, Serialize)]
enum PlotType {
    P(Option<FourBar>, Vec<LineData<2>>),
    S(Option<SFourBar>, Vec<LineData<3>>),
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
            (io::Curve::P(c), Self::P(_, curves)) => curves.push(LineData::new(s, c)),
            (io::Curve::P(c), p @ Self::S(_, _)) => *p = Self::P(None, vec![LineData::new(s, c)]),
            (io::Curve::S(c), p @ Self::P(_, _)) => *p = Self::S(None, vec![LineData::new(s, c)]),
            (io::Curve::S(c), Self::S(_, curves)) => curves.push(LineData::new(s, c)),
        }
    }

    fn show(&mut self, ui: &mut Ui, i: usize) {
        if match self {
            Self::P(_, c) => c.is_empty(),
            Self::S(_, c) => c.is_empty(),
        } {
            return;
        }
        ui.group(|ui| {
            let mut j = 0;
            match self {
                Self::P(_, c) => c.retain_mut(|data| {
                    j += 1;
                    data.show(ui, Id::new("style").with(i).with(j))
                }),
                Self::S(_, c) => c.retain_mut(|data| {
                    j += 1;
                    data.show(ui, Id::new("style").with(i).with(j))
                }),
            }
        });
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct PlotOpt {
    plot: Rc<RefCell<PlotType>>,
    angle: Option<f64>,
    opt: fb_plot::Opt<'static>,
}

impl PlotOpt {
    fn show(&mut self, ui: &mut Ui, lnk: &mut super::link::Linkages, i: usize) -> bool {
        let keep = ui
            .horizontal(|ui| {
                match &*self.plot.borrow() {
                    PlotType::P(_, _) => ui.heading(format!("Planar Plot {{{i}}}")),
                    PlotType::S(_, _) => ui.heading(format!("Spherical Plot {{{i}}}")),
                };
                !ui.button("✖").clicked()
            })
            .inner;
        ui.collapsing("Linkage", |ui| {
            if self.plot.borrow().has_fb() {
                check_on(ui, "Input angle", &mut self.angle, angle_f);
                if ui.button("✖ Remove Linkage").clicked() {
                    self.plot.borrow_mut().remove_fb();
                    self.angle.take();
                }
            } else {
                ui.label("No Linkage");
            }
            ui.horizontal(|ui| {
                if ui.button("🖴 Load from").clicked() {
                    if let Some((angle, fb)) = lnk.projs.current_fb_state() {
                        self.plot.borrow_mut().set_fb(fb);
                        self.angle.replace(angle);
                    }
                }
                lnk.projs.select(ui, false);
            });
            if ui.button("🖴 Load from RON").clicked() {
                let plot = self.plot.clone();
                io::open_ron(move |_, fb| plot.borrow_mut().set_fb(fb));
            }
            if let PlotType::S(Some(fb), _) = &mut *self.plot.borrow_mut() {
                if ui.button("⚾ Take Sphere").clicked() {
                    *fb = fb.take_sphere();
                    self.angle.take();
                }
            }
        });
        ui.collapsing("Curves", |ui| {
            self.plot.borrow_mut().show(ui, i);
            ui.horizontal(|ui| {
                if ui.button("🖴 Add from").clicked() {
                    if let Some(c) = lnk.projs.current_curve() {
                        self.plot.borrow_mut().push_fb_curve("New Curve", c);
                    }
                }
                lnk.projs.select(ui, false);
            });
            if ui.button("🖴 Add from CSV").clicked() {
                let plot = self.plot.clone();
                io::open_csv(move |_, c| plot.borrow_mut().push_fb_curve("New Curve", c));
            }
            if ui.button("🖴 Add from RON").clicked() {
                let res = lnk.cfg.res;
                let plot = self.plot.clone();
                io::open_ron(move |_, fb| {
                    plot.borrow_mut()
                        .push_fb_curve("New Curve", fb.into_curve(res));
                });
            }
        });
        ui.collapsing("Plot Option", |ui| {
            nonzero_i(ui, "Stroke size: ", &mut self.opt.stroke, 1);
            nonzero_i(ui, "Font size: ", &mut self.opt.font, 1);
            check_on(ui, "Font Family", &mut self.opt.font_family, |ui, s| {
                ui.text_edit_singleline(s.to_mut())
            });
            ui.checkbox(&mut self.opt.grid, "Show grid");
            ui.checkbox(&mut self.opt.axis, "Show axis");
            ui.horizontal(|ui| {
                ui.label("Legend");
                const LIST: [fb_plot::LegendPos; 10] = fb_plot::LegendPos::LIST;
                combo_enum(ui, "legend", &mut self.opt.legend, LIST, |e| e.name());
            });
        });
        keep
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Plotter {
    size: u32,
    shape: (usize, usize),
    current: usize,
    queue: Vec<PlotOpt>,
}

impl Default for Plotter {
    fn default() -> Self {
        Self {
            size: 800,
            shape: (1, 1),
            current: 0,
            queue: Vec::new(),
        }
    }
}

impl Plotter {
    pub(crate) fn show(&mut self, ui: &mut Ui, lnk: &mut super::link::Linkages) {
        ui.heading("Plotter");
        nonzero_i(ui, "Subplot size: ", &mut self.size, 1);
        ui.horizontal(|ui| {
            ui.label("Width:");
            counter(ui, &mut self.shape.1, 1..=10);
        });
        ui.horizontal(|ui| {
            ui.label("Height:");
            counter(ui, &mut self.shape.0, 1..=10);
        });
        let cap = self.shape.0 * self.shape.1;
        ui.label(format!("Capacity: {}/{cap}", self.queue.len()));
        // Grid view
        Grid::new("plot-grid").show(ui, |ui| {
            for i in 0..self.shape.0 {
                for j in 0..self.shape.1 {
                    let n = i * self.shape.1 + j;
                    let checked = self.current == n;
                    if ui.selectable_label(checked, format!("{{{n}}}")).clicked() {
                        self.current = n;
                    }
                }
                ui.end_row();
            }
        });
        // Subplot settings
        let mut i = 0;
        self.queue.retain_mut(|opt| {
            i += 1;
            ui.group(|ui| opt.show(ui, lnk, i)).inner
        });
        if ui.button("⊞ Add Subplot").clicked() {
            self.queue.push(PlotOpt::default());
        }
        ui.separator();
        if ui.button("💾 Save Plot").clicked() {
            if cap == self.queue.len() {
                self.save_plot();
            } else {
                io::alert(format!("Incorrect plot number: {}/{cap}", self.queue.len()));
            }
        }
    }

    fn save_plot(&mut self) {
        use four_bar::plot::IntoDrawingArea as _;
        let mut buf = String::new();
        let size = (
            self.size * self.shape.1 as u32,
            self.size * self.shape.0 as u32,
        );
        let b = fb_plot::SVGBackend::with_string(&mut buf, size);
        b.into_drawing_area()
            .split_evenly(self.shape)
            .into_iter()
            .zip(&self.queue)
            .for_each(|(root, p_opt)| match &*p_opt.plot.borrow() {
                PlotType::P(fb, c) => {
                    let mut fig = plot2d::Figure::new_ref(fb.as_ref()).with_opt(p_opt.opt.clone());
                    if let Some(angle) = p_opt.angle {
                        fig = fig.angle(angle);
                    }
                    for data in c {
                        let (label, line, style, color) = data.share();
                        fig = fig.add_line(label, line, style, color);
                    }
                    fig.plot(root).alert("Plot");
                }
                PlotType::S(fb, c) => {
                    let mut fig = plot3d::Figure::new_ref(fb.as_ref()).with_opt(p_opt.opt.clone());
                    if let Some(angle) = p_opt.angle {
                        fig = fig.angle(angle);
                    }
                    for data in c {
                        let (label, line, style, color) = data.share();
                        fig = fig.add_line(label, line, style, color);
                    }
                    fig.plot(root).alert("Plot");
                }
            });
        io::save_svg_ask(&buf, "figure.svg");
    }
}
