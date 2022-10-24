use super::{io, linkages::Linkages, widgets::unit};
use crate::csv::{dump_csv, parse_csv};
use eframe::egui::*;
use four_bar::{cb::Codebook, curve, efd, mh, syn};
use serde::{Deserialize, Serialize};
use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        Arc, RwLock,
    },
};

mod painting;

fn parse_curve(s: &str) -> Option<Vec<[f64; 2]>> {
    if let Ok(curve) = parse_csv(s) {
        // CSV
        Some(curve)
    } else if let Ok(curve) = ron::from_str::<Vec<Vec<f64>>>(s) {
        // Nested array
        Some(curve.into_iter().map(|c| [c[0], c[1]]).collect())
    } else if let Ok(curve) = ron::from_str(s) {
        // Tuple array
        Some(curve)
    } else {
        None
    }
}

#[inline]
fn ron_pretty<S: ?Sized + Serialize>(s: &S) -> String {
    ron::ser::to_string_pretty(s, Default::default()).unwrap()
}

fn solve<S>(task: &Task, cb: &Codebook, config: SynConfig, setting: S) -> four_bar::FourBar
where
    S: mh::Setting,
    S::Algorithm: mh::Algorithm<syn::PathSyn>,
{
    #[cfg(target_arch = "wasm32")]
    use instant::Instant;
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::Instant;
    let start_time = Instant::now();
    let SynConfig { method: _, gen, pop, mode, target } = config;
    let mut s =
        four_bar::mh::Solver::build(setting, syn::PathSyn::from_curve(&target, mode).unwrap());
    if let Some(candi) = matches!(mode, syn::Mode::Closed | syn::Mode::Open)
        .then(|| cb.fetch_raw(&target, pop))
        .filter(|candi| !candi.is_empty())
    {
        s = s.pop_num(candi.len());
        let fitness = candi.iter().map(|(f, _)| *f).collect();
        let pool = candi
            .into_iter()
            .map(|(_, fb)| fb.to_norm().vec())
            .collect::<Vec<_>>();
        s = s.pool_and_fitness(mh::ndarray::arr2(&pool), fitness);
    } else {
        s = s.pop_num(pop);
    }
    let (_, fb) = s
        .task(|ctx| ctx.gen == gen || !task.start.load(Ordering::Relaxed))
        .callback(|ctx| {
            task.conv.write().unwrap().push(ctx.best_f);
            task.gen.store(ctx.gen, Ordering::Relaxed);
            let time = (Instant::now() - start_time).as_secs();
            task.time.store(time, Ordering::Relaxed);
        })
        .solve()
        .unwrap()
        .result();
    fb
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Synthesis {
    config: UiConfig,
    #[serde(skip)]
    cb: Arc<RwLock<Codebook>>,
    tasks: Vec<Task>,
    csv_open: bool,
    conv_open: bool,
    plot_linkage: bool,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct UiConfig {
    painting: painting::Painting,
    syn: SynConfig,
    curve_str: Arc<RwLock<String>>,
    #[serde(skip)]
    changed: Arc<AtomicU8>,
    #[serde(skip)]
    efd_h: Option<usize>,
    #[serde(skip)]
    paint_input: bool,
}

impl PartialEq for UiConfig {
    fn eq(&self, other: &Self) -> bool {
        self.syn == other.syn && *self.curve_str.read().unwrap() == *other.curve_str.read().unwrap()
    }
}

impl UiConfig {
    const TICK: u8 = 30;

    fn init_poll(&mut self) {
        let curve_str = self.curve_str.read().unwrap();
        if self.syn.target.is_empty() && !curve_str.is_empty() {
            if let Some(curve) = parse_curve(&curve_str) {
                self.efd_h = efd::fourier_power_nyq(&curve);
                self.syn.target = curve;
            }
        }
    }

    fn poll(&mut self) {
        if self.changed.load(Ordering::Relaxed) >= Self::TICK {
            if let Some(curve) = parse_curve(&self.curve_str.read().unwrap()) {
                self.efd_h = efd::fourier_power_nyq(&curve);
                self.syn.target = curve;
            }
            self.changed.store(0, Ordering::Relaxed);
        }
        self.changed.fetch_add(1, Ordering::Relaxed);
    }

    fn set_target(&mut self, target: Vec<[f64; 2]>) {
        *self.curve_str.write().unwrap() = dump_csv(&target).unwrap();
        self.efd_h = efd::fourier_power_nyq(&target);
        self.syn.target = target;
    }

    fn write_curve_str(&self, f: impl FnOnce(&[[f64; 2]]) -> String) {
        *self.curve_str.write().unwrap() = f(&self.syn.target);
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.label("Support CSV or RON array only.");
        ui.horizontal(|ui| {
            if ui.button("🖴 Open Curves").clicked() {
                let curve_csv = self.curve_str.clone();
                let changed = self.changed.clone();
                io::open_csv_single(move |_, s| {
                    *curve_csv.write().unwrap() = s;
                    changed.store(UiConfig::TICK, Ordering::Relaxed);
                });
            }
            if ui.button("💾 Save CSV").clicked() {
                io::save_csv_ask(&self.syn.target);
            }
            if ui.button("🗑 Clear").clicked() {
                self.curve_str.write().unwrap().clear();
            }
        });
        let mode = &mut self.syn.mode;
        ui.radio_value(mode, syn::Mode::Closed, "Closed path matching");
        ui.radio_value(mode, syn::Mode::Partial, "Closed path match open path");
        ui.radio_value(mode, syn::Mode::Open, "Open path matching");
        ui.label("Transform:");
        ui.horizontal_wrapped(|ui| {
            if ui.button("🔀 To CSV").clicked() {
                self.write_curve_str(|c| dump_csv(c).unwrap());
            }
            if ui.button("🔀 To array of tuple").clicked() {
                self.write_curve_str(ron_pretty);
            }
            if ui.button("🔀 To array of array").clicked() {
                self.write_curve_str(|c| {
                    let c = c.iter().copied().map(Vec::from).collect::<Vec<_>>();
                    ron_pretty(&c)
                });
            }
            if let Some(h) = self.efd_h {
                if ui.button(format!("🔀 Re-describe ({h})")).clicked() {
                    self.write_curve_str(|c| {
                        let c = self.syn.mode.regularize(c);
                        let len = c.len();
                        let efd = efd::Efd2::from_curve_harmonic(c, h).unwrap();
                        dump_csv(curve::remove_last(efd.generate(len))).unwrap()
                    });
                }
                if ui.button("🔀 Reverse").clicked() {
                    self.write_curve_str(|c| dump_csv(c.iter().rev().collect::<Vec<_>>()).unwrap());
                }
            }
        });
        ui.separator();
        ui.radio_value(&mut self.paint_input, false, "Input with text");
        ui.radio_value(&mut self.paint_input, true, "Input with painting");
        if !self.paint_input {
            ui.label("Past curve data here:");
            ScrollArea::both().show(ui, |ui| {
                let mut s = self.curve_str.write().unwrap();
                let w = TextEdit::multiline(&mut *s)
                    .code_editor()
                    .desired_width(f32::INFINITY);
                ui.add(w);
            });
        } else if self.painting.ui(ui, &mut self.syn.target).changed() {
            self.efd_h = efd::fourier_power_nyq(&self.syn.target);
            *self.curve_str.write().unwrap() = dump_csv(&self.syn.target).unwrap();
        }
        self.poll();
        self.changed.fetch_add(1, Ordering::Relaxed);
        ui.ctx().request_repaint();
    }
}

#[derive(Default, Deserialize, Serialize, Clone, PartialEq)]
enum Method {
    #[default]
    De,
    Fa,
    Pso,
    Rga,
    Tlbo,
}

impl Method {
    const fn name(&self) -> &'static str {
        match self {
            Method::De => "Differential Evolution",
            Method::Fa => "Firefly Algorithm",
            Method::Pso => "Particle Swarm Optimization",
            Method::Rga => "Real-coded Genetic Algorithm",
            Method::Tlbo => "Teaching Learning Based Optimization",
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
struct SynConfig {
    method: Method,
    gen: u64,
    pop: usize,
    mode: syn::Mode,
    #[serde(skip)]
    target: Vec<[f64; 2]>,
}

impl Default for SynConfig {
    fn default() -> Self {
        Self {
            method: Method::default(),
            gen: 50,
            pop: 400,
            mode: syn::Mode::Closed,
            target: Vec::new(),
        }
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(default)]
struct Task {
    #[serde(skip)]
    start: Arc<AtomicBool>,
    #[serde(
        serialize_with = "super::atomic::serialize_u64",
        deserialize_with = "super::atomic::deserialize_u64"
    )]
    gen: Arc<AtomicU64>,
    total_gen: u64,
    #[serde(
        serialize_with = "super::atomic::serialize_u64",
        deserialize_with = "super::atomic::deserialize_u64"
    )]
    time: Arc<AtomicU64>,
    conv: Arc<RwLock<Vec<f64>>>,
}

impl Synthesis {
    pub(crate) fn show(&mut self, ui: &mut Ui, linkage: &mut Linkages) {
        ui.horizontal(|ui| {
            ui.heading("Synthesis");
            reset_button(ui, &mut self.config);
        });
        let method = &mut self.config.syn.method;
        ui.horizontal_wrapped(|ui| {
            for (m, abb) in [
                (Method::De, "DE"),
                (Method::Fa, "FA"),
                (Method::Pso, "PSO"),
                (Method::Rga, "RGA"),
                (Method::Tlbo, "TLBO"),
            ] {
                let name = m.name();
                ui.selectable_value(method, m, abb).on_hover_text(name);
            }
        });
        ui.horizontal_wrapped(|ui| {
            let url = match method {
                Method::De => "https://en.wikipedia.org/wiki/Differential_evolution",
                Method::Fa => "https://en.wikipedia.org/wiki/Firefly_algorithm",
                Method::Pso => "https://en.wikipedia.org/wiki/Particle_swarm_optimization",
                Method::Rga => "https://en.wikipedia.org/wiki/Genetic_algorithm",
                Method::Tlbo => "https://doi.org/10.1016/j.cad.2010.12.015",
            };
            ui.hyperlink_to(method.name(), url)
                .on_hover_text(format!("More about {}", method.name()));
        });
        unit(ui, "Generation: ", &mut self.config.syn.gen, 1);
        unit(ui, "Population: ", &mut self.config.syn.pop, 1);
        ui.label("Edit target curve then click refresh button to update the task.");
        if ui.button("🛠 Target Curve").clicked() {
            self.csv_open = !self.csv_open;
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("Codebook");
            if ui.button("Reset").clicked() {
                self.cb.write().unwrap().clear();
            }
        });
        ui.label("Use pre-searched dataset to increase the speed.");
        let size = self.cb.read().unwrap().size();
        ui.label(format!("Number of data: {size}"));
        if ui.button("🖴 Open").clicked() {
            let cb = self.cb.clone();
            io::open_cb(move |a| {
                let cb_new = Codebook::read(Cursor::new(a)).unwrap();
                cb.write()
                    .unwrap()
                    .merge_inplace(&cb_new)
                    .unwrap_or_default();
            })
        }
        ui.separator();
        ui.heading("Optimization");
        if ui.button("📉 Convergence Plot").clicked() {
            self.conv_open = !self.conv_open;
        }
        self.tasks.retain(|task| {
            let mut keep = true;
            ui.horizontal(|ui| {
                let start = task.start.load(Ordering::Relaxed);
                if start {
                    if ui.small_button("⏹").clicked() {
                        task.start.store(false, Ordering::Relaxed);
                    }
                } else {
                    if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                        keep = false;
                    }
                    if ui
                        .small_button("💾")
                        .on_hover_text("Save history plot")
                        .clicked()
                    {
                        io::save_history_ask(&task.conv.read().unwrap(), "history.svg");
                    }
                }
                ui.label(format!("{}s", task.time.load(Ordering::Relaxed)));
                let pb = task.gen.load(Ordering::Relaxed) as f32 / task.total_gen as f32;
                ui.add(ProgressBar::new(pb).show_percentage().animate(start));
            });
            keep
        });
        #[cfg(target_arch = "wasm32")]
        ui.label("WARNING: Web platform will freeze UI when start solving!");
        ui.horizontal(|ui| {
            let enabled = !self.config.syn.target.is_empty();
            if ui.add_enabled(enabled, Button::new("▶ Start")).clicked() {
                self.start_syn(linkage.projs.queue());
            }
            ui.add(ProgressBar::new(0.).show_percentage());
        });
        ui.separator();
        ui.heading("Projects");
        ui.label("Compare results from a project's coupler curve.");
        if linkage.projs.select(ui, false) {
            ui.horizontal(|ui| {
                if ui.button("💾 Save Comparison").clicked() {
                    let target = &self.config.syn.target;
                    let curve = linkage.projs.current_curve();
                    let opt = self
                        .plot_linkage
                        .then(|| linkage.projs.four_bar_state().use_dot(linkage.cfg.plot_dot));
                    io::save_curve_ask(target, &curve, opt, "fb.svg");
                }
                ui.checkbox(&mut self.plot_linkage, "With linkage");
            });
            if ui.button("🗐 Copy Coupler Curve").clicked() {
                self.config.set_target(linkage.projs.current_curve());
            }
        }
        self.convergence_plot(ui);
        self.target_curve_editor(ui);
    }

    fn convergence_plot(&mut self, ui: &mut Ui) {
        Window::new("📉 Convergence Plot")
            .open(&mut self.conv_open)
            .show(ui.ctx(), |ui| {
                plot::Plot::new("plot_conv")
                    .legend(Default::default())
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_scroll(false)
                    .show(ui, |ui| {
                        for (i, task) in self.tasks.iter().enumerate() {
                            let conv = task.conv.read().unwrap();
                            let pts1 = plot::PlotPoints::from_ys_f64(&conv);
                            let pts2 = plot::PlotPoints::from_ys_f64(&conv);
                            let name = format!("Task {}", i + 1);
                            ui.line(plot::Line::new(pts1).fill(-1.5).name(&name));
                            ui.points(plot::Points::new(pts2).name(name).stems(0.));
                        }
                    });
            });
    }

    fn target_curve_editor(&mut self, ui: &mut Ui) {
        self.config.init_poll();
        Window::new("🛠 Target Curve")
            .open(&mut self.csv_open)
            .vscroll(false)
            .show(ui.ctx(), |ui| self.config.ui(ui));
    }

    pub(crate) fn plot(&self, ui: &mut plot::PlotUi) {
        if !self.config.syn.target.is_empty() {
            let line = plot::Line::new(self.config.syn.target.clone())
                .name("Synthesis target")
                .style(plot::LineStyle::dashed_loose())
                .width(3.);
            ui.line(line);
        }
    }

    fn start_syn(&mut self, queue: super::proj::Queue) {
        use four_bar::mh::methods::*;
        #[cfg(not(target_arch = "wasm32"))]
        use four_bar::mh::rayon::spawn;
        let config = self.config.syn.clone();
        let task = Task {
            total_gen: config.gen,
            start: Arc::new(AtomicBool::new(true)),
            ..Task::default()
        };
        self.tasks.push(task.clone());
        let cb = self.cb.clone();
        let f = move || {
            let cb = cb.read().unwrap();
            let fb = match config.method {
                Method::De => solve(&task, &cb, config, De::default()),
                Method::Fa => solve(&task, &cb, config, Fa::default()),
                Method::Pso => solve(&task, &cb, config, Pso::default()),
                Method::Rga => solve(&task, &cb, config, Rga::default()),
                Method::Tlbo => solve(&task, &cb, config, Tlbo::default()),
            };
            queue.push(None, fb);
            task.start.store(false, Ordering::Relaxed);
        };
        #[cfg(not(target_arch = "wasm32"))]
        spawn(f);
        #[cfg(target_arch = "wasm32")]
        f(); // Block
    }
}
