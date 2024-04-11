use super::widgets::*;
use crate::io;
use eframe::egui::*;
use four_bar::{mech, plot as fb_plot};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell, rc::Rc};

fn fig_ui<M, const D: usize>(
    ui: &mut Ui,
    fig: &mut Rc<RefCell<fb_plot::FigureBase<'static, 'static, M, [f64; D]>>>,
    lnk: &mut super::link::Linkages,
    get_fb: impl Fn(io::Fb) -> Option<M> + Copy + 'static,
    get_curve: impl Fn(io::Curve) -> Option<Vec<[f64; D]>> + Copy + 'static,
    to_fb: impl Fn(M) -> io::Fb + Copy + 'static,
) where
    M: Clone + mech::CurveGen<D>,
{
    ui.collapsing("Linkage", |ui| {
        ui.horizontal(|ui| {
            let mut fig = fig.borrow_mut();
            if let Some(fb) = &fig.fb {
                if ui.button("✚ Export").clicked() {
                    lnk.projs.push_fb(to_fb(fb.clone().into_owned()));
                }
                if ui.button("✖ Remove").clicked() {
                    fig.fb = None;
                }
            } else {
                ui.label("No linkage loaded");
            }
        });
        ui.horizontal(|ui| {
            if let Some(fb) = lnk.projs.current_fb_state().and_then(|(_, fb)| get_fb(fb)) {
                if ui.button("🖴 Load from").clicked() {
                    fig.borrow_mut().fb = Some(Cow::Owned(fb));
                }
            } else {
                ui.add_enabled(false, Button::new("🖴 Load from"));
            }
            lnk.projs.select(ui);
        });
        if ui.button("🖴 Load from RON").clicked() {
            let fig = fig.clone();
            io::open_ron_single(move |_, fb| {
                io::alert!(
                    ("Wrong linkage type", get_fb(fb)),
                    ("*", |fb| fig.borrow_mut().fb = Some(Cow::Owned(fb))),
                );
            });
        }
    });
    ui.collapsing("Curves", |ui| {
        fig.borrow_mut()
            .retain_lines(|i, line| ui.group(|ui| fig_line_ui(ui, i, line)).inner);
        const NEW_CURVE: &str = "New Curve";
        ui.horizontal(|ui| {
            if let Some(c) = lnk.projs.current_curve().and_then(get_curve) {
                if ui.button("🖴 Add from").clicked() {
                    fig.borrow_mut().push_line_default(NEW_CURVE, c);
                }
            } else {
                ui.add_enabled(false, Button::new("🖴 Load from"));
            }
            lnk.projs.select(ui);
        });
        if ui.button("🖴 Add from CSV").clicked() {
            let fig = fig.clone();
            io::open_csv(move |_, c| {
                io::alert!(
                    ("Wrong curve type", get_curve(c)),
                    ("*", |c| fig.borrow_mut().push_line_default(NEW_CURVE, c)),
                );
            });
        }
        if ui.button("🖴 Add from RON (360pt)").clicked() {
            let fig = fig.clone();
            io::open_ron(move |_, fb| {
                io::alert!(
                    ("Wrong linkage type", get_fb(fb)),
                    ("*", |fb| {
                        fig.borrow_mut().push_line_default(NEW_CURVE, fb.curve(360));
                    })
                );
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
        ui.checkbox(&mut line.mk_fp, "Mark the first point");
    });
    keep
}

#[derive(Deserialize, Serialize, Clone)]
enum PlotType {
    P(Rc<RefCell<fb_plot::fb::Figure<'static, 'static>>>),
    S(Rc<RefCell<fb_plot::sfb::Figure<'static, 'static>>>),
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
                    io::Fb::P(fb) => Some(fb),
                    _ => None,
                };
                let get_curve = |c| match c {
                    io::Curve::P(c) => Some(c),
                    _ => None,
                };
                fig_ui(ui, fig, lnk, get_fb, get_curve, io::Fb::P);
            }
            PlotType::S(fig) => {
                ui.heading("Spherical Plot");
                if let Some(fb) = &mut fig.borrow_mut().fb {
                    if ui
                        .button("⚾ Take Sphere")
                        .on_hover_text("Draw the sphere without the linkage")
                        .clicked()
                    {
                        match fb {
                            Cow::Borrowed(src) => *fb = Cow::Owned(src.clone().take_sphere()),
                            Cow::Owned(fb) => fb.take_sphere_inplace(),
                        }
                    }
                }
                let get_fb = |fb| match fb {
                    io::Fb::S(fb) => Some(fb),
                    _ => None,
                };
                let get_curve = |c| match c {
                    io::Curve::S(c) => Some(c),
                    _ => None,
                };
                fig_ui(ui, fig, lnk, get_fb, get_curve, io::Fb::S);
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
                    let text = if self.queue[n].is_none() {
                        format!("{{{n}}}?")
                    } else {
                        format!("{{{n}}}")
                    };
                    if ui.selectable_label(self.curr == n, text).clicked() {
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
                self.queue[self.curr] = None;
            }
        } else {
            ui.heading("Empty Plot");
            ui.horizontal(|ui| {
                if ui.button("✚ Planar").clicked() {
                    self.queue[self.curr] = Some(PlotType::new_p());
                }
                if ui.button("✚ Spatial").clicked() {
                    self.queue[self.curr] = Some(PlotType::new_s());
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
                            *curr = Some(plot.clone());
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
        use std::iter::zip;
        let mut buf = String::new();
        let size = (
            self.size * self.shape.1 as u32,
            self.size * self.shape.0 as u32,
        );
        let b = fb_plot::SVGBackend::with_string(&mut buf, size);
        for (root, p_opt) in zip(b.into_drawing_area().split_evenly(self.shape), &self.queue) {
            match &p_opt {
                None => (),
                Some(PlotType::P(fig)) => io::alert!("Plot", fig.borrow().plot(root)),
                Some(PlotType::S(fig)) => io::alert!("Plot", fig.borrow().plot(root)),
            }
        }
        io::save_svg_ask(&buf, "figure.svg");
    }
}
