use super::widgets::*;
use crate::io;
use eframe::egui::*;
use four_bar::{mech, plot as fb_plot};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    iter::zip,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering::{Relaxed, SeqCst},
        },
        Arc, Mutex,
    },
};

const GIF_RES: usize = 60;
const NEW_CURVE: &str = "New Curve";
type Fig<M, const D: usize> = fb_plot::FigureBase<'static, 'static, M, [f64; D]>;

fn fig_ui<M, const D: usize>(
    ui: &mut Ui,
    fig: &mut Arc<Mutex<Fig<M, D>>>,
    lnk: &mut super::link::Linkages,
    get_fb: impl Fn(io::Fb) -> Option<M> + Copy + 'static,
    get_curve: impl Fn(&mut Fig<M, D>, io::Curve) + 'static,
    to_fb: impl Fn(M) -> io::Fb + Copy + 'static,
) where
    M: Clone + mech::CurveGen<D>,
{
    ui.collapsing("Linkage", |ui| {
        ui.horizontal(|ui| {
            let mut fig = fig.lock().unwrap();
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
                    fig.lock().unwrap().fb = Some(Cow::Owned(fb));
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
                    ("*", |fb| fig.lock().unwrap().fb = Some(Cow::Owned(fb))),
                );
            });
        }
    });
    ui.collapsing("Curves", |ui| {
        fig.lock()
            .unwrap()
            .retain_lines(|i, line| ui.group(|ui| fig_line_ui(ui, i, line)).inner);
        ui.horizontal(|ui| {
            if let Some(c) = lnk.projs.current_curve() {
                if ui.button("🖴 Add from").clicked() {
                    get_curve(&mut *fig.lock().unwrap(), c);
                }
            } else {
                ui.add_enabled(false, Button::new("🖴 Load from"));
            }
            lnk.projs.select(ui);
        });
        if ui.button("🖴 Add from CSV").clicked() {
            let fig = fig.clone();
            io::open_csv(move |_, c| {
                io::alert!("Wrong curve type", get_curve(&mut *fig.lock().unwrap(), c));
            });
        }
        if ui.button("🖴 Add from RON (360pt)").clicked() {
            let fig = fig.clone();
            io::open_ron(move |_, fb| {
                io::alert!(
                    ("Wrong linkage type", get_fb(fb)),
                    ("*", |fb| {
                        fig.lock()
                            .unwrap()
                            .push_line_default(NEW_CURVE, fb.curve(360));
                    })
                );
            });
        }
    });
    ui.collapsing("Plot Option", |ui| {
        let mut fig = fig.lock().unwrap();
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
            match &mut line.line {
                fb_plot::LineType::Line(..) => _ = ui.label("[Line]"),
                fb_plot::LineType::Pose { is_frame, .. } => {
                    ui.label("[Pose]");
                    ui.checkbox(is_frame, "Frame Style");
                }
            };
            ui.with_layout(Layout::right_to_left(Align::LEFT), |ui| {
                !ui.button("✖").clicked()
            })
            .inner
        })
        .inner;
    ui.text_edit_singleline(line.label.to_mut());
    ui.horizontal(|ui| {
        ui.label("Style");
        let id = Id::new("sty").with(i);
        combo_enum(ui, id, &mut line.style, fb_plot::Style::LIST, |e| e.name());
    });
    ui.horizontal(|ui| {
        let color = &mut line.color.color;
        {
            let mut buf = [color.0, color.1, color.2];
            ui.color_edit_button_srgb(&mut buf);
            [color.0, color.1, color.2] = buf;
        }
        any_i(ui, &mut color.0);
        any_i(ui, &mut color.1);
        any_i(ui, &mut color.2);
        ui.checkbox(&mut line.color.filled, "Filled");
    });
    keep
}

#[derive(Deserialize, Serialize, Clone)]
enum PlotType {
    P(Arc<Mutex<fb_plot::fb::Figure<'static, 'static>>>),
    S(Arc<Mutex<fb_plot::sfb::Figure<'static, 'static>>>),
}

impl PlotType {
    fn new_p() -> Self {
        Self::new_p_and_get().0
    }

    fn new_p_and_get() -> (Self, Arc<Mutex<fb_plot::fb::Figure<'static, 'static>>>) {
        let fig = Arc::new(Mutex::new(Default::default()));
        (Self::P(fig.clone()), fig)
    }

    fn new_s() -> Self {
        Self::S(Default::default())
    }

    fn new_s_and_get() -> (Self, Arc<Mutex<fb_plot::sfb::Figure<'static, 'static>>>) {
        let fig = Arc::new(Mutex::new(Default::default()));
        (Self::S(fig.clone()), fig)
    }

    fn show(&mut self, ui: &mut Ui, lnk: &mut super::link::Linkages) {
        match self {
            PlotType::P(fig) => {
                ui.heading("Planar Plot");
                let get_fb = |fb| match fb {
                    io::Fb::P(fb) => Some(fb),
                    io::Fb::M(mfb) => Some(mfb.into_fb()),
                    _ => None,
                };
                let get_curve = |fig: &mut Fig<_, 2>, c| match c {
                    io::Curve::P(c) => fig.push_line_default(NEW_CURVE, c),
                    io::Curve::M(c) => {
                        let (c, v) = c.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
                        fig.push_pose_default(NEW_CURVE, (c, v, 1.), false);
                    }
                    _ => (),
                };
                fig_ui(ui, fig, lnk, get_fb, get_curve, io::Fb::P);
            }
            PlotType::S(fig) => {
                ui.heading("Spherical Plot");
                if let Some(fb) = &mut fig.lock().unwrap().fb {
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
                let get_curve = |fig: &mut Fig<_, 3>, c| {
                    if let io::Curve::S(c) = c {
                        fig.push_line_default(NEW_CURVE, c);
                    }
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
    #[serde(skip)]
    gif_pg: Option<Arc<AtomicUsize>>,
    #[serde(skip)]
    gif_queue: Arc<mutex::Mutex<Vec<u8>>>,
}

impl Default for Plotter {
    fn default() -> Self {
        Self {
            size: 1600,
            shape: (1, 1),
            curr: 0,
            queue: vec![None],
            gif_pg: None,
            gif_queue: Default::default(),
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
                    PlotType::P(fig) => io::save_ron_ask(&*fig.lock().unwrap(), name, |_| ()),
                    PlotType::S(fig) => io::save_ron_ask(&*fig.lock().unwrap(), name, |_| ()),
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
                    let (plot, fig) = PlotType::new_p_and_get();
                    self.queue[self.curr] = Some(plot);
                    io::open_ron(move |_, cfg| *fig.lock().unwrap() = cfg);
                }
                if ui.button("🖴 Load Spherical").clicked() {
                    let (plot, fig) = PlotType::new_s_and_get();
                    self.queue[self.curr] = Some(plot);
                    io::open_ron(move |_, cfg| *fig.lock().unwrap() = cfg);
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
        ui.horizontal(|ui| {
            if let Some(pg) = &self.gif_pg {
                let pg_value = pg.load(Relaxed) as f32 / GIF_RES as f32;
                if small_btn(ui, "⏹", "Stop") {
                    pg.store(GIF_RES, SeqCst);
                    self.gif_pg = None;
                }
                ui.add(ProgressBar::new(pg_value).show_percentage().animate(true));
                let mut queue = self.gif_queue.lock();
                if !queue.is_empty() {
                    io::save_gif_ask(std::mem::take(&mut queue), "figure.gif");
                    self.gif_pg = None;
                }
            }
        });
        if self.queue.iter().all(Option::is_none) {
            return;
        }
        ui.horizontal(|ui| {
            if ui.button("💾 Save Plot").clicked() {
                self.save_plot();
            }
            if ui
                .add_enabled(self.gif_pg.is_none(), Button::new("🎥 Save GIF Plot"))
                .clicked()
            {
                self.save_plot_gif();
            }
        });
    }

    fn save_plot(&mut self) {
        use four_bar::plot::IntoDrawingArea as _;
        let mut buf = String::new();
        let size = (
            self.size * self.shape.1 as u32,
            self.size * self.shape.0 as u32,
        );
        let b = fb_plot::SVGBackend::with_string(&mut buf, size);
        for (root, p_opt) in zip(b.into_drawing_area().split_evenly(self.shape), &self.queue) {
            match &p_opt {
                None => (),
                Some(PlotType::P(fig)) => io::alert!("Plot", fig.lock().unwrap().plot(root)),
                Some(PlotType::S(fig)) => io::alert!("Plot", fig.lock().unwrap().plot(root)),
            }
        }
        io::save_svg_ask(&buf, "figure.svg");
    }

    fn save_plot_gif(&mut self) {
        use four_bar::plot::IntoDrawingArea as _;
        use image::{codecs::gif, DynamicImage, Frame, RgbImage};
        let pg = Arc::new(AtomicUsize::new(0));
        self.gif_pg = Some(pg.clone());
        let queue = self.gif_queue.clone();
        let fig_queue = self.queue.clone();
        let shape = self.shape;
        let size = (self.size * shape.1 as u32, self.size * shape.0 as u32);
        let f = move || {
            let mut buf = Vec::new();
            let mut w = gif::GifEncoder::new_with_speed(&mut buf, 30);
            io::alert!("Plot", w.set_repeat(gif::Repeat::Infinite));
            for curr in 0..GIF_RES {
                if pg.load(SeqCst) == GIF_RES {
                    return;
                }
                let mut frame = vec![0; size.0 as usize * size.1 as usize * 3];
                let b = fb_plot::BitMapBackend::with_buffer(&mut frame, size);
                for (root, p_opt) in zip(b.into_drawing_area().split_evenly(shape), &fig_queue) {
                    match &p_opt {
                        None => (),
                        Some(PlotType::P(fig)) => {
                            io::alert!("Plot", fig.lock().unwrap().plot_video(root, curr, GIF_RES));
                        }
                        Some(PlotType::S(fig)) => {
                            io::alert!("Plot", fig.lock().unwrap().plot_video(root, curr, GIF_RES));
                        }
                    }
                }
                let image = RgbImage::from_vec(size.0, size.1, frame).unwrap();
                io::alert!(
                    "Plot",
                    w.encode_frame(Frame::new(DynamicImage::from(image).into_rgba8()))
                );
                pg.store(curr, Relaxed);
            }
            drop(w);
            *queue.lock() = buf;
        };
        #[cfg(not(target_arch = "wasm32"))]
        four_bar::mh::rayon::spawn(f);
        #[cfg(target_arch = "wasm32")]
        f(); // Block
    }
}
