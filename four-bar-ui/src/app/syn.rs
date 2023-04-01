use super::{io, link::Linkages, widgets::*};
use crate::syn_cmd::SynCmd;
use eframe::egui::*;
use four_bar::{cb::FbCodebook, csv::*, *};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, RwLock,
};

const CLOSED_URL: &str =
    "https://drive.google.com/file/d/1xOgypg2fCWgfAPVneuDO-cnPdc_GHCsK/view?usp=sharing";
const OPEN_URL: &str =
    "https://drive.google.com/file/d/1vPPK4KzyAiaC25m1MiJGiSpxbl9ZDEW4/view?usp=sharing";

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

fn solve<S>(task: &Task, cb: &FbCodebook, config: SynConfig, setting: S) -> four_bar::FourBar
where
    S: mh::Setting,
{
    #[cfg(target_arch = "wasm32")]
    use instant::Instant;
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::Instant;
    let start_time = Instant::now();
    let SynConfig { gen, pop, mode, target } = config;
    let mut s = four_bar::mh::Solver::build(
        setting,
        syn2d::PlanarSyn::from_curve(&target, mode).unwrap(),
    );
    if let Some(candi) = matches!(mode, syn2d::Mode::Closed | syn2d::Mode::Open)
        .then(|| cb.fetch_raw(&target, pop))
        .filter(|candi| !candi.is_empty())
    {
        s = s.pop_num(candi.len());
        let fitness = candi.iter().map(|(f, _)| *f).collect();
        let pool = candi
            .into_iter()
            .map(|(_, fb)| fb.as_array())
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
    cfg: UiConfig,
    #[serde(skip)]
    cb: Arc<RwLock<FbCodebook>>,
    tasks: Vec<Task>,
    // plot with linkage
    plot_linkage: bool,
    // competitor
    cpt: usize,
    #[serde(skip)]
    tmp_target: Arc<RwLock<Vec<[f64; 2]>>>,
    #[serde(skip)]
    conv_open: bool,
    #[serde(skip)]
    from_plot_open: bool,
}

#[derive(Default, Deserialize, Serialize, PartialEq)]
#[serde(default)]
struct UiConfig {
    method: SynCmd,
    syn: SynConfig,
    #[serde(skip)]
    efd_h: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
struct SynConfig {
    gen: u64,
    pop: usize,
    mode: syn2d::Mode,
    target: Vec<[f64; 2]>,
}

impl Default for SynConfig {
    fn default() -> Self {
        Self {
            gen: 50,
            pop: 200,
            mode: syn2d::Mode::Closed,
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
    pub(crate) fn show(&mut self, ui: &mut Ui, lnk: &mut Linkages) {
        ui.horizontal(|ui| {
            ui.heading("Synthesis");
            reset_button(ui, &mut self.cfg);
        });
        ui.group(|ui| self.opt_setting(ui));
        unit(ui, "Generation: ", &mut self.cfg.syn.gen, 1);
        unit(ui, "Population: ", &mut self.cfg.syn.pop, 1);
        ui.separator();
        ui.heading("Target Curve");
        if !self.tmp_target.read().unwrap().is_empty() {
            let mut g = self.tmp_target.write().unwrap();
            std::mem::swap(&mut self.cfg.syn.target, &mut g);
        }
        toggle_btn(ui, &mut self.from_plot_open, "🖊 Add from canvas")
            .on_hover_text("Click canvas to add target point drictly!");
        ui.horizontal(|ui| {
            if ui.button("🖴 Load").clicked() {
                let target = self.tmp_target.clone();
                io::open_csv_single(move |_, s| {
                    if let Some(c) = parse_curve(&s) {
                        *target.write().unwrap() = c;
                    }
                });
            }
            if ui.button("💾 Save CSV").clicked() {
                io::save_csv_ask(&self.cfg.syn.target);
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("🗐 Copy CSV").clicked() {
                ui.output_mut(|s| s.copied_text = dump_csv(&self.cfg.syn.target).unwrap());
            }
            if ui.button("🗐 Copy Array of Tuple").clicked() {
                ui.output_mut(|s| s.copied_text = ron_pretty(&self.cfg.syn.target));
            }
            if ui.button("🗐 Copy Array of Array").clicked() {
                let c = self
                    .cfg
                    .syn
                    .target
                    .iter()
                    .map(|c| Vec::from(*c))
                    .collect::<Vec<_>>();
                ui.output_mut(|s| s.copied_text = ron_pretty(&c));
            }
        });
        ui.group(|ui| table(ui, &mut self.cfg.syn.target));
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("Codebook");
            if ui.button("Reset").clicked() {
                self.cb.write().unwrap().clear();
            }
        });
        ui.label("Use pre-searched dataset to increase the speed.");
        ui.label(format!("Number of data: {}", self.cb.read().unwrap().len()));
        ui.collapsing("Help", |ui| {
            ui.label("Run \"four-bar cb\" in command line window to generate codebook file.");
            ui.horizontal(|ui| {
                ui.label("Author provided:");
                url_btn(ui, "", "Downlod closed curve dataset", CLOSED_URL);
                url_btn(ui, "", "Downlod open curve dataset", OPEN_URL);
            });
        });
        ui.horizontal(|ui| {
            if ui.button("🖴 Load").clicked() {
                let cb = self.cb.clone();
                io::open_cb(move |cb_new| {
                    cb.write()
                        .unwrap()
                        .merge_inplace(&cb_new)
                        .unwrap_or_default();
                });
            }
            if ui.button("🗑 Clear").clicked() {
                self.cb.write().unwrap().clear();
            }
        });
        ui.separator();
        ui.heading("Optimization");
        toggle_btn(ui, &mut self.conv_open, "📉 Convergence Plot");
        self.tasks.retain(|task| {
            let mut keep = true;
            ui.horizontal(|ui| {
                let start = task.start.load(Ordering::Relaxed);
                if start {
                    if small_btn(ui, "⏹", "Stop") {
                        task.start.store(false, Ordering::Relaxed);
                    }
                } else {
                    if small_btn(ui, "🗑", "Delete") {
                        keep = false;
                    }
                    if small_btn(ui, "💾", "Save history plot") {
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
            let enabled = !self.cfg.syn.target.is_empty();
            if ui.add_enabled(enabled, Button::new("▶ Start")).clicked() {
                self.start_syn(lnk.projs.queue());
            }
            ui.add(ProgressBar::new(0.).show_percentage());
        });
        ui.separator();
        ui.heading("Projects");
        ui.label("Compare results from a project's coupler curve.");
        if lnk.projs.select(ui, false) {
            let len = lnk.projs.len() + 1;
            if self.cpt >= len {
                self.cpt = len - 1;
            }
            ComboBox::from_label("").show_index(ui, &mut self.cpt, len, |i| {
                if i == 0 {
                    "None".to_string()
                } else {
                    lnk.projs[i - 1].name()
                }
            });
            ui.horizontal(|ui| {
                if ui.button("💾 Save Comparison").clicked() {
                    let mut curves = vec![
                        ("Target", self.cfg.syn.target.clone()),
                        ("Synthesized", lnk.projs.current_curve()),
                    ];
                    if self.cpt > 0 {
                        let curve = lnk.projs[self.cpt - 1].clone_curve();
                        curves.push(("Competitor", curve));
                    }
                    let (fb, angle) = lnk.projs.four_bar_state();
                    let opt = if self.plot_linkage {
                        plot2d::Opt::from(&fb)
                            .angle(angle)
                            .inner(lnk.cfg.plot.clone())
                    } else {
                        plot2d::Opt::new().inner(lnk.cfg.plot.clone())
                    };
                    let curves = curves.iter().map(|(s, c)| (*s, c.as_slice()));
                    io::save_curve_ask(curves, opt, "fb.svg");
                }
                ui.checkbox(&mut self.plot_linkage, "With linkage");
            });
            if ui.button("🗐 Copy Coupler Curve").clicked() {
                self.cfg.syn.target = lnk.projs.current_curve();
            }
        }
        self.convergence_plot(ui);
    }

    fn opt_setting(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for &(name, abbr, f) in SynCmd::LIST {
                let c = self.cfg.method.abbr() == abbr;
                if ui.selectable_label(c, abbr).on_hover_text(name).clicked() && !c {
                    self.cfg.method = f();
                }
            }
        });
        let m = &mut self.cfg.method;
        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to(m.name(), m.link())
                .on_hover_text(format!("More about {}", m.name()));
        });
        macro_rules! param {
            ($s:ident, $($name:ident),+) => {{$(
                percent(ui, concat![stringify!($name), ": "], &mut $s.$name);
            )+}};
        }
        match m {
            SynCmd::De(s) => {
                ui.horizontal_wrapped(|ui| {
                    use mh::de::Strategy::*;
                    for (i, strategy) in [S1, S2, S3, S4, S5, S6, S7, S8, S9, S10]
                        .into_iter()
                        .enumerate()
                    {
                        ui.selectable_value(&mut s.strategy, strategy, format!("S{i}"));
                    }
                });
                param!(s, f, cross);
            }
            SynCmd::Fa(s) => param!(s, alpha, beta_min, gamma),
            SynCmd::Pso(s) => param!(s, cognition, social, velocity),
            SynCmd::Rga(s) => param!(s, cross, mutate, win, delta),
            SynCmd::Tlbo(_) => (),
        }
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

    pub(crate) fn plot(&mut self, ui: &mut plot::PlotUi) {
        if self.from_plot_open && ui.plot_clicked() {
            // Add target curve from canvas
            let plot::PlotPoint { x, y } = ui.pointer_coordinate().unwrap();
            self.cfg.syn.target.push([x, y]);
        }
        if !self.cfg.syn.target.is_empty() {
            const NAME: &str = "Synthesis target";
            let line = plot::Line::new(self.cfg.syn.target.clone())
                .name(NAME)
                .style(plot::LineStyle::dashed_loose())
                .width(3.);
            ui.line(line);
            let points = plot::Points::new(self.cfg.syn.target.clone())
                .name(NAME)
                .filled(false)
                .radius(5.);
            ui.points(points);
        }
    }

    fn start_syn(&mut self, queue: super::proj::Queue) {
        #[cfg(not(target_arch = "wasm32"))]
        use four_bar::mh::rayon::spawn;
        let method = self.cfg.method.clone();
        let config = self.cfg.syn.clone();
        let task = Task {
            total_gen: config.gen,
            start: Arc::new(AtomicBool::new(true)),
            ..Task::default()
        };
        self.tasks.push(task.clone());
        let cb = self.cb.clone();
        let f = move || {
            let cb = cb.read().unwrap();
            let fb = match method {
                SynCmd::De(s) => solve(&task, &cb, config, s),
                SynCmd::Fa(s) => solve(&task, &cb, config, s),
                SynCmd::Pso(s) => solve(&task, &cb, config, s),
                SynCmd::Rga(s) => solve(&task, &cb, config, s),
                SynCmd::Tlbo(s) => solve(&task, &cb, config, s),
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
