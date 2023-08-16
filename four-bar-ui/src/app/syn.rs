use super::{link::Linkages, widgets::*};
use crate::{io, io::Alert as _, syn_cmd, syn_cmd::Target};
use eframe::egui::*;
use four_bar::{cb, csv, efd, mh, syn};
use serde::{Deserialize, Serialize};
use smartcore::linalg::basic::arrays::Array;
use std::{borrow::Cow, sync::Arc};

#[inline]
fn ron_pretty<S: ?Sized + Serialize>(s: &S) -> String {
    ron::ser::to_string_pretty(s, Default::default()).unwrap()
}

#[derive(Deserialize, Serialize, Clone)]
struct Task {
    time: std::time::Duration,
    conv: Vec<f64>,
}

#[derive(Default)]
enum Cache {
    #[default]
    Empty,
    Curve(io::Curve),
    Cb(io::Cb),
}

struct CbVis {
    pt: [f64; 2],
    is_open: bool,
    is_sphere: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct CbCfg {
    size: usize,
    harmonic: usize,
    is_open: bool,
}

impl Default for CbCfg {
    fn default() -> Self {
        Self { size: 10000, harmonic: 20, is_open: false }
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Synthesis {
    method: syn_cmd::SynMethod,
    cfg: syn_cmd::SynConfig,
    cb_cfg: CbCfg,
    target: io::Curve,
    tasks: Vec<Task>,
    #[serde(skip)]
    cb: io::CbPool,
    #[serde(skip)]
    cb_vis: Vec<CbVis>,
    #[serde(skip)]
    cb_pg: Option<Arc<std::sync::atomic::AtomicU32>>,
    #[serde(skip)]
    queue: Arc<mutex::RwLock<Cache>>,
    #[serde(skip)]
    task_queue: Vec<Arc<mutex::RwLock<(f32, Task)>>>,
    #[serde(skip)]
    conv_open: bool,
    #[serde(skip)]
    cb_vis_open: bool,
    #[serde(skip)]
    from_plot_open: bool,
}

impl Synthesis {
    pub(crate) fn show(&mut self, ui: &mut Ui, lnk: &mut Linkages) {
        ui.horizontal(|ui| {
            ui.heading("Synthesis");
            reset_button(ui, &mut self.cfg);
        });
        ui.collapsing("Method", |ui| {
            ui.group(|ui| self.opt_setting(ui));
            check_on(ui, "Random seed", &mut self.cfg.seed, any_i);
            nonzero_i(ui, "Generation: ", &mut self.cfg.gen, 1);
            nonzero_i(ui, "Population: ", &mut self.cfg.pop, 1);
            nonzero_i(ui, "Resolution: ", &mut self.cfg.res, 1);
        });
        ui.collapsing("Codebook", |ui| self.cb_setting(ui));
        ui.separator();
        match self.target {
            io::Curve::P(_) => ui.heading("Planar Target Curve"),
            io::Curve::S(_) => ui.heading("Spherical Target Curve"),
        };
        ui.horizontal(|ui| {
            ui.label("Mode: ");
            for (mode, name) in [
                (syn::Mode::Closed, "Closed"),
                (syn::Mode::Open, "Open"),
                (syn::Mode::Partial, "Partial"),
            ] {
                ui.selectable_value(&mut self.cfg.mode, mode, name);
            }
        });
        match std::mem::replace(&mut *self.queue.write(), Cache::Empty) {
            Cache::Curve(curve) => self.target = curve,
            Cache::Cb(cb) => self.cb.merge_inplace(cb).alert("Merge Codebook"),
            Cache::Empty => (),
        }
        toggle_btn(ui, &mut self.from_plot_open, "🖊 Add from canvas")
            .on_hover_text("Click canvas to add target point drictly!");
        ui.horizontal(|ui| {
            if ui.button("🖴 Add from").clicked() {
                if let Some(target) = lnk.projs.current_curve() {
                    self.target = target;
                }
            }
            lnk.projs.select(ui, false);
        });
        ui.horizontal(|ui| {
            if ui.button("🖴 Add from CSV").clicked() {
                let queue = self.queue.clone();
                io::open_csv_single(move |_, c| *queue.write() = Cache::Curve(c));
            }
            if ui.button("💾 Save CSV").clicked() {
                match &self.target {
                    io::Curve::P(t) => io::save_csv_ask(t),
                    io::Curve::S(t) => io::save_csv_ask(t),
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("🗐 Copy CSV").clicked() {
                let text = match &self.target {
                    io::Curve::P(t) => csv::dump_csv(t).unwrap(),
                    io::Curve::S(t) => csv::dump_csv(t).unwrap(),
                };
                ui.output_mut(|s| s.copied_text = text);
            }
            if ui.button("🗐 Copy Array of Tuple").clicked() {
                let text = match &self.target {
                    io::Curve::P(t) => ron_pretty(t),
                    io::Curve::S(t) => ron_pretty(t),
                };
                ui.output_mut(|s| s.copied_text = text);
            }
            if ui.button("🗐 Copy Array of Array").clicked() {
                macro_rules! vec_nest {
                    ($iter:ident) => {
                        $iter.iter().map(|c| c.to_vec()).collect::<Vec<_>>()
                    };
                }
                let text = match &self.target {
                    io::Curve::P(t) => ron_pretty(&vec_nest!(t)),
                    io::Curve::S(t) => ron_pretty(&vec_nest!(t)),
                };
                ui.output_mut(|s| s.copied_text = text);
            }
        });
        match &mut self.target {
            io::Curve::P(t) => table(ui, t),
            io::Curve::S(t) => table(ui, t),
        }
        ui.separator();
        ui.heading("Optimization");
        toggle_btn(ui, &mut self.conv_open, "📉 Convergence Plot");
        self.tasks.retain(|task| {
            ui.horizontal(|ui| {
                let keep = !small_btn(ui, "🗑", "Delete");
                if small_btn(ui, "💾", "Save history plot") {
                    io::save_history_ask(&task.conv, "history.svg");
                }
                ui.label(format!("{:.4?}", task.time));
                ui.colored_label(Color32::GREEN, "Finished");
                keep
            })
            .inner
        });
        self.task_queue.retain(|task| {
            let (pg, task) = &mut *task.write();
            ui.horizontal(|ui| {
                if small_btn(ui, "⏹", "Stop") {
                    *pg = 1.;
                }
                ui.label(format!("{:.4?}", task.time));
                ui.add(ProgressBar::new(*pg).show_percentage().animate(true));
            });
            // FIXME: Use `drain_filter`
            if *pg == 1. {
                self.tasks.push(task.clone());
                false
            } else {
                true
            }
        });
        #[cfg(target_arch = "wasm32")]
        ui.colored_label(Color32::RED, "Web version freezes UI when solving starts!");
        ui.horizontal(|ui| {
            let enabled = !self.target.is_empty();
            if ui.add_enabled(enabled, Button::new("▶ Start")).clicked() {
                self.start_syn(lnk);
            }
            ui.add(ProgressBar::new(0.).show_percentage());
        });
        self.cb_vis(ui);
        self.convergence_plot(ui);
    }

    fn opt_setting(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for &(name, abbr, f) in syn_cmd::SynMethod::LIST {
                let c = self.method.abbr() == abbr;
                if ui.selectable_label(c, abbr).on_hover_text(name).clicked() && !c {
                    self.method = f();
                }
            }
        });
        let m = &mut self.method;
        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to(m.name(), m.link())
                .on_hover_text(format!("More about {}", m.name()));
        });
        macro_rules! param {
            ($s:ident, $($name:ident),+) => {{$(
                percent(ui, concat![stringify!($name), ": "], &mut $s.$name);
            )+}};
        }
        use syn_cmd::SynMethod::*;
        match m {
            De(s) => {
                use mh::de::Strategy::*;
                const ST: [mh::de::Strategy; 10] = [S1, S2, S3, S4, S5, S6, S7, S8, S9, S10];
                combo_enum(ui, "de strategy", &mut s.strategy, ST, |e| {
                    let i = ST.iter().position(|s| s == e).unwrap() + 1;
                    format!("S{i}")
                });
                param!(s, f, cross);
            }
            Fa(s) => param!(s, alpha, beta_min, gamma),
            Pso(s) => param!(s, cognition, social, velocity),
            Rga(s) => param!(s, cross, mutate, win, delta),
            Tlbo(_) => (),
        }
    }

    fn cb_setting(&mut self, ui: &mut Ui) {
        ui.label("Use pre-searched dataset to increase the speed.");
        ui.label(format!("No. of planar data: {}", self.cb.as_fb().len()));
        ui.label(format!("No. of spherical data: {}", self.cb.as_sfb().len()));
        ui.horizontal(|ui| {
            if ui.button("🖴 Load").clicked() {
                let queue = self.queue.clone();
                io::open_cb(move |cb| *queue.write() = Cache::Cb(cb));
            }
            if ui.button("💾 Planar").clicked() {
                io::save_cb_ask(self.cb.as_fb());
            }
            if ui.button("💾 Spherical").clicked() {
                io::save_cb_ask(self.cb.as_sfb());
            }
        });
        ui.separator();
        nonzero_i(ui, "Size: ", &mut self.cb_cfg.size, 1);
        nonzero_i(ui, "Harmonic: ", &mut self.cb_cfg.harmonic, 1);
        ui.checkbox(&mut self.cb_cfg.is_open, "Is open curve");
        ui.horizontal(|ui| {
            macro_rules! impl_make_cb {
                ($cb:ident, $cb_ty:ident) => {
                    let size = self.cb_cfg.size;
                    let cfg = cb::Cfg::new()
                        .res(self.cfg.res)
                        .size(size)
                        .harmonic(self.cb_cfg.harmonic)
                        .is_open(self.cb_cfg.is_open);
                    let queue = self.queue.clone();
                    let pg = Arc::new(std::sync::atomic::AtomicU32::new(0));
                    self.cb_pg.replace(pg.clone());
                    let f = move || {
                        let cb = cb::$cb::make_with(cfg, |p| {
                            let p = p as f32 / size as f32;
                            pg.store(p.to_bits(), std::sync::atomic::Ordering::Relaxed);
                        });
                        *queue.write() = Cache::Cb(io::Cb::$cb_ty(cb));
                    };
                    #[cfg(not(target_arch = "wasm32"))]
                    mh::rayon::spawn(f);
                    #[cfg(target_arch = "wasm32")]
                    f(); // Block
                };
            }
            if self.cb_pg.is_none() {
                if ui.button("✚ Planar").clicked() {
                    impl_make_cb!(FbCodebook, P);
                }
                if ui.button("✚ Spherical").clicked() {
                    impl_make_cb!(SFbCodebook, S);
                }
            }
        });
        if let Some(pg) = &self.cb_pg {
            let pg = f32::from_bits(pg.load(std::sync::atomic::Ordering::Relaxed));
            ui.add(ProgressBar::new(pg).show_percentage().animate(true));
            if pg == 1. {
                self.cb_pg = None;
            }
        }
        ui.group(|ui| {
            if ui.button("☁ Visualize").clicked() {
                if !self.cb_vis_open {
                    self.cb_vis_cache();
                } else {
                    self.cb_vis.clear();
                    self.cb_vis.shrink_to_fit();
                }
                self.cb_vis_open = !self.cb_vis_open;
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("🗑 Clear Planar").clicked() {
                self.cb.as_fb_mut().clear();
            }
            if ui.button("🗑 Clear Spherical").clicked() {
                self.cb.as_sfb_mut().clear();
            }
        });
    }

    fn convergence_plot(&mut self, ui: &mut Ui) {
        Window::new("📉 Convergence Plot")
            .open(&mut self.conv_open)
            .show(ui.ctx(), |ui| {
                static_plot("plot_conv").show(ui, |ui| {
                    for (i, task) in self.tasks.iter().enumerate() {
                        let pts1 = plot::PlotPoints::from_ys_f64(&task.conv);
                        let pts2 = plot::PlotPoints::from_ys_f64(&task.conv);
                        let name = format!("Task {}", i + 1);
                        ui.line(plot::Line::new(pts1).fill(-1.5).name(&name));
                        ui.points(plot::Points::new(pts2).name(name).stems(0.));
                    }
                });
            });
    }

    pub(crate) fn plot(&mut self, ui: &mut plot::PlotUi, lnk: &Linkages) {
        if !self.target.is_empty() {
            const NAME: &str = "Synthesis target";
            let target = match &self.target {
                io::Curve::P(t) => t.clone(),
                io::Curve::S(t) => t.iter().map(|[x, y, _]| [*x, *y]).collect(),
            };
            let line = plot::Line::new(target.clone())
                .name(NAME)
                .style(plot::LineStyle::dashed_loose())
                .width(3.);
            ui.line(line);
            let points = plot::Points::new(target)
                .name(NAME)
                .filled(false)
                .radius(5.);
            ui.points(points);
        }
        if !self.from_plot_open || !ui.plot_clicked() {
            return;
        }
        // Add target curve from canvas
        let p = ui.pointer_coordinate().unwrap();
        match &mut self.target {
            io::Curve::P(t) => t.push([p.x, p.y]),
            io::Curve::S(t) => {
                let f = || {
                    let [sx, sy, sz, r] = lnk.projs.current_sphere()?;
                    let dx = p.x - sx;
                    let dy = p.y - sy;
                    (dx.hypot(dy) <= r).then_some([p.x, p.y, r * r - dx * dx - dy * dy + sz])
                };
                if let Some(c) = f() {
                    t.push(c);
                } else {
                    let p = plot::Points::new([p.x, p.y])
                        .shape(plot::MarkerShape::Cross)
                        .color(Color32::RED)
                        .radius(30.);
                    ui.points(p);
                }
            }
        }
    }

    fn cb_vis_cache(&mut self) {
        // Cache the visualization of codebook
        fn t_sne<C, D, const N: usize>(cb: &cb::Codebook<C, D, N>, is_sphere: bool) -> Vec<CbVis>
        where
            C: cb::Code<D, N> + Send,
            D: efd::EfdDim,
        {
            use smartcore::{decomposition::pca::*, linalg::basic::matrix::DenseMatrix};
            let data = cb
                .fb_iter()
                .zip(cb.open_iter())
                .flat_map(|((buf, inv), open)| {
                    [buf.as_slice(), &[inv as u8 as f64, open as u8 as f64]].concat()
                })
                .collect::<Vec<_>>();
            let data = DenseMatrix::new(cb.len(), N + 2, data, false);
            let pca = PCA::fit(&data, Default::default()).unwrap();
            let reduced = pca.transform(&data).unwrap();
            cb.fb_iter()
                .zip(cb.open_iter())
                .enumerate()
                .map(|(i, (_, is_open))| CbVis {
                    pt: [*reduced.get((i, 0)), *reduced.get((i, 1))],
                    is_open,
                    is_sphere,
                })
                .collect()
        }
        self.cb_vis
            .reserve(self.cb.as_fb().len() + self.cb.as_sfb().len());
        if !self.cb.as_fb().is_empty() {
            self.cb_vis.extend(t_sne(self.cb.as_fb(), false));
        }
        if !self.cb.as_sfb().is_empty() {
            self.cb_vis.extend(t_sne(self.cb.as_sfb(), true));
        }
    }

    fn cb_vis(&mut self, ui: &mut Ui) {
        if !self.cb_vis_open {
            return;
        }
        // FIXME: GUI error
        Window::new("☁ Codebook Visualization")
            .open(&mut self.cb_vis_open)
            .fixed_size([600., 600.])
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    let mut f = |name, draw_sphere| {
                        static_plot(name).data_aspect(1.).show(ui, |ui| {
                            for &CbVis { pt, is_open, is_sphere } in &self.cb_vis {
                                if is_sphere != draw_sphere {
                                    continue;
                                }
                                let (name, color) = if is_open {
                                    ("Open Curve", Color32::RED)
                                } else {
                                    ("Closed Curve", Color32::BLUE)
                                };
                                ui.points(plot::Points::new(pt).color(color).name(name));
                            }
                        });
                    };
                    f("plot_cb_vis_planar", false);
                    // f("plot_cb_vis_spherical", true);
                });
            });
    }

    fn start_syn(&mut self, lnk: &Linkages) {
        #[cfg(target_arch = "wasm32")]
        use instant::Instant;
        #[cfg(not(target_arch = "wasm32"))]
        use std::time::Instant;
        let task = Task {
            time: std::time::Duration::from_secs(0),
            conv: Vec::new(),
        };
        let task = Arc::new(mutex::RwLock::new((0., task)));
        self.task_queue.push(task.clone());
        let method = self.method.clone();
        let target = match self.target.clone() {
            io::Curve::P(t) => Target::P(t.into(), Cow::Owned(self.cb.as_fb().clone())),
            io::Curve::S(t) => Target::S(t.into(), Cow::Owned(self.cb.as_sfb().clone())),
        };
        let cfg = self.cfg.clone();
        let total_gen = self.cfg.gen;
        let queue = lnk.projs.queue();
        let f = move || {
            let t0 = Instant::now();
            let s = syn_cmd::Solver::new(method, target, cfg, move |best_f, gen| {
                let (pg, task) = &mut *task.write();
                *pg = gen as f32 / total_gen as f32;
                task.conv.push(best_f);
                task.time = t0.elapsed();
            });
            s.solve()
                .alert_then("Initialization", |fb| queue.push(None, fb));
        };
        #[cfg(not(target_arch = "wasm32"))]
        mh::rayon::spawn(f);
        #[cfg(target_arch = "wasm32")]
        f(); // Block
    }
}
