use super::widgets::*;
use crate::io::{self, Alert as _};
use eframe::egui::*;
use four_bar::{efd, fb, plot as fb_plot};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell, rc::Rc};

fn fig_ui<D, M, const N: usize>(
    ui: &mut Ui,
    fig: &mut Rc<RefCell<fb_plot::FigureBase<'static, 'static, M, [f64; N]>>>,
    lnk: &mut super::link::Linkages,
    get_fb: impl Fn(io::Fb) -> Option<M> + Copy + 'static,
    get_curve: impl Fn(io::Curve) -> Option<Vec<[f64; N]>> + Copy + 'static,
) where
    D: efd::EfdDim,
    D::Trans: efd::Trans<Coord = [f64; N]>,
    M: Clone + fb::CurveGen<D>,
{
    ui.collapsing("Linkage", |ui| {
        if fig.borrow().fb.is_some() {
            if ui.button("✖ Remove Linkage").clicked() {
                fig.borrow_mut().fb.take();
            }
        } else {
            ui.label("No linkage loaded");
        }
        ui.horizontal(|ui| {
            let state = lnk.projs.current_fb_state().and_then(|(_, fb)| get_fb(fb));
            if let Some(fb) = state {
                if ui.button("🖴 Load from").clicked() {
                    fig.borrow_mut().fb.replace(Cow::Owned(fb));
                }
            } else {
                ui.add_enabled(false, Button::new("🖴 Load from"));
            }
            lnk.projs.select(ui);
        });
        if ui.button("🖴 Load from RON").clicked() {
            let fig = fig.clone();
            io::open_ron_single(move |_, fb| {
                let done = |fb| _ = fig.borrow_mut().fb.replace(Cow::Owned(fb));
                get_fb(fb).alert_then("Wrong linkage type", done);
            });
        }
    });
    ui.collapsing("Curves", |ui| {
        {
            let mut i = 0;
            fig.borrow_mut().lines.retain_mut(|line| {
                ui.group(|ui| {
                    i += 1;
                    fig_line_ui(ui, i, &mut line.borrow_mut())
                })
                .inner
            });
        }
        ui.horizontal(|ui| {
            if let Some(c) = lnk.projs.current_curve().and_then(get_curve) {
                if ui.button("🖴 Add from").clicked() {
                    fig.borrow_mut().push_line_default("New Curve", c);
                }
            } else {
                ui.add_enabled(false, Button::new("🖴 Load from"));
            }
            lnk.projs.select(ui);
        });
        if ui.button("🖴 Add from CSV").clicked() {
            let fig = fig.clone();
            io::open_csv(move |_, c| {
                let done = |c| {
                    let mut fig = fig.borrow_mut();
                    fig.push_line_default("New Curve", c);
                };
                get_curve(c).alert_then("Wrong curve type", done);
            });
        }
        if ui.button("🖴 Add from RON").clicked() {
            let res = lnk.cfg.res;
            let fig = fig.clone();
            io::open_ron(move |_, fb| {
                get_fb(fb).alert_then("Wrong linkage type", |fb| {
                    fig.borrow_mut()
                        .push_line_default("New Curve", fb.curve(res));
                });
            });
        }
    });
    ui.collapsing("Plot Option", |ui| {
        let mut fig = fig.borrow_mut();
        nonzero_i(ui, "Stroke size: ", &mut fig.stroke, 1);
        nonzero_i(ui, "Font size: ", &mut fig.font, 1);
        check_on(ui, "Font Family", &mut fig.font_family, |ui, s| {
            ui.text_edit_singleline(s.to_mut())
        });
        ui.checkbox(&mut fig.grid, "Show grid");
        ui.checkbox(&mut fig.axis, "Show axis");
        ui.horizontal(|ui| {
            use fb_plot::LegendPos;
            ui.label("Legend");
            combo_enum(ui, "legend", &mut fig.legend, LegendPos::LIST, |e| e.name());
        });
    });
}

fn fig_line_ui<const N: usize>(
    ui: &mut Ui,
    i: usize,
    line: &mut fb_plot::LineData<[f64; N]>,
) -> bool {
    let keep = ui
        .horizontal(|ui| {
            ui.text_edit_singleline(line.label.to_mut());
            !ui.button("✖").clicked()
        })
        .inner;
    ui.horizontal(|ui| {
        ui.label("Style");
        let id = Id::new("sty").with(i);
        combo_enum(ui, id, &mut line.style, fb_plot::Style::LIST, |e| e.name());
    });
    ui.horizontal(|ui| {
        ui.color_edit_button_srgb(&mut line.color);
        any_i(ui, &mut line.color[0]);
        any_i(ui, &mut line.color[1]);
        any_i(ui, &mut line.color[2]);
        ui.checkbox(&mut line.filled, "Filled");
    });
    keep
}

#[derive(Deserialize, Serialize, Clone)]
enum PlotType {
    P(Rc<RefCell<fb_plot::plot2d::Figure<'static, 'static>>>),
    S(Rc<RefCell<fb_plot::plot3d::Figure<'static, 'static>>>),
}

impl PlotType {
    fn new_p() -> Self {
        Self::P(Default::default())
    }

    fn new_s() -> Self {
        Self::S(Default::default())
    }

    fn show(&mut self, ui: &mut Ui, lnk: &mut super::link::Linkages) {
        match self {
            PlotType::P(fig) => {
                ui.heading("Planar Plot");
                let get_fb = |fb| match fb {
                    io::Fb::Fb(fb) => Some(fb),
                    _ => None,
                };
                let get_curve = |c| match c {
                    io::Curve::P(c) => Some(c),
                    _ => None,
                };
                fig_ui(ui, fig, lnk, get_fb, get_curve);
            }
            PlotType::S(fig) => {
                ui.heading("Spherical Plot");
                {
                    let mut fig = fig.borrow_mut();
                    if let Some(fb) = &mut fig.fb {
                        if ui
                            .button("⚾ Take Sphere")
                            .on_hover_text("Draw the sphere without the linkage")
                            .clicked()
                        {
                            *fb = Cow::Owned(fb.take_sphere());
                        }
                    }
                }
                let get_fb = |fb| match fb {
                    io::Fb::SFb(fb) => Some(fb),
                    _ => None,
                };
                let get_curve = |c| match c {
                    io::Curve::S(c) => Some(c),
                    _ => None,
                };
                fig_ui(ui, fig, lnk, get_fb, get_curve);
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Plotter {
    size: u32,
    shape: (usize, usize),
    queue: Vec<Option<PlotType>>,
    #[serde(skip)]
    curr: usize,
}

impl Default for Plotter {
    fn default() -> Self {
        Self {
            size: 1600,
            shape: (1, 1),
            curr: 0,
            queue: vec![None],
        }
    }
}

impl Plotter {
    pub(crate) fn show(&mut self, ui: &mut Ui, lnk: &mut super::link::Linkages) {
        ui.heading("Plotter");
        nonzero_i(ui, "Subplot size: ", &mut self.size, 1);
        ui.horizontal(|ui| {
            ui.label("Width:");
            if counter(ui, &mut self.shape.1, 1..=10).clicked() {
                self.queue.resize_with(self.shape.0 * self.shape.1, || None);
                self.curr = self.curr.clamp(0, self.queue.len() - 1);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Height:");
            if counter(ui, &mut self.shape.0, 1..=10).clicked() {
                self.queue.resize_with(self.shape.0 * self.shape.1, || None);
                self.curr = self.curr.clamp(0, self.queue.len() - 1);
            }
        });
        // Grid view
        Grid::new("plot-grid").show(ui, |ui| {
            for i in 0..self.shape.0 {
                for j in 0..self.shape.1 {
                    let n = i * self.shape.1 + j;
                    let checked = self.curr == n;
                    let mut text = format!("{{{n}}}");
                    if self.queue[n].is_none() {
                        text += "*";
                    }
                    if ui.selectable_label(checked, text).clicked() {
                        self.curr = n;
                    }
                }
                ui.end_row();
            }
        });
        // Subplot settings
        if let Some(plot) = &mut self.queue[self.curr] {
            plot.show(ui, lnk);
            if ui.button("💾 Save Plot Settings").clicked() {
                let name = "plot.fig.ron";
                match plot {
                    PlotType::P(fig) => io::save_ron_ask(&*fig.borrow(), name, |_| ()),
                    PlotType::S(fig) => io::save_ron_ask(&*fig.borrow(), name, |_| ()),
                }
            }
            if ui.button("✖ Delete Plot").clicked() {
                self.queue[self.curr].take();
            }
        } else {
            ui.heading("Empty Plot");
            ui.horizontal(|ui| {
                if ui.button("✚ Planar").clicked() {
                    self.queue[self.curr].replace(PlotType::new_p());
                }
                if ui.button("✚ Spatial").clicked() {
                    self.queue[self.curr].replace(PlotType::new_s());
                }
            });
            ui.horizontal(|ui| {
                if ui.button("🖴 Load Planar").clicked() {
                    let PlotType::P(fig) = self.queue[self.curr].insert(PlotType::new_p()) else {
                        unreachable!()
                    };
                    let fig = fig.clone();
                    io::open_ron(move |_, cfg| *fig.borrow_mut() = cfg);
                }
                if ui.button("🖴 Load Spherical").clicked() {
                    let PlotType::S(fig) = self.queue[self.curr].insert(PlotType::new_s()) else {
                        unreachable!()
                    };
                    let fig = fig.clone();
                    io::open_ron(move |_, cfg| *fig.borrow_mut() = cfg);
                }
            });
            ui.menu_button("Copy From ⏷", |ui| {
                let rng = (0..self.curr).chain(self.curr..self.queue.len());
                let (front, last) = self.queue.split_at_mut(self.curr);
                let (curr, last) = last.split_first_mut().unwrap();
                let mut is_empty = true;
                for (i, plot) in rng.zip(front.iter().chain(last.iter())) {
                    if let Some(plot) = plot {
                        if ui.button(format!("{{{i}}}")).clicked() {
                            curr.replace(plot.clone());
                        }
                        is_empty = false;
                    }
                }
                if is_empty {
                    ui.label("No Target");
                }
            });
        }
        ui.separator();
        if ui.button("💾 Save Plot").clicked() {
            self.save_plot();
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
            .for_each(|(root, p_opt)| match &p_opt {
                None => (),
                Some(PlotType::P(fig)) => fig.borrow().plot(root).alert("Plot"),
                Some(PlotType::S(fig)) => fig.borrow().plot(root).alert("Plot"),
            });
        io::save_svg_ask(&buf, "figure.svg");
    }
}
