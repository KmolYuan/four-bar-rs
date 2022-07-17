use super::{io, linkages::Linkages, widgets::unit};
use crate::{as_values::as_values, dump_csv, parse_csv};
use eframe::egui::*;
use four_bar::{curve, syn::Mode};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, RwLock,
    },
};

const ERR_DES: &str = "This error is calculated with point by point strategy.\n\
    Increase resolution for more accurate calculations.";

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

fn write_curve_str(s: &RwLock<String>, f: impl FnOnce(Vec<[f64; 2]>) -> String) {
    let curve = parse_curve(&s.read().unwrap());
    if let Some(curve) = curve {
        *s.write().unwrap() = f(curve);
    }
}

#[inline]
fn ron_pretty(s: impl Serialize) -> String {
    ron::ser::to_string_pretty(&s, Default::default()).unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
fn solve<S>(task: &Task, config: SynConfig, setting: S) -> four_bar::FourBar
where
    S: four_bar::mh::Setting,
    S::Algorithm: four_bar::mh::Algorithm<four_bar::syn::Task>,
{
    use std::time::Instant;
    let start_time = Instant::now();
    four_bar::mh::Solver::build(setting)
        .pop_num(config.pop)
        .task(|ctx| ctx.gen == config.gen || !task.start.load(Ordering::Relaxed))
        .callback(|ctx| {
            task.conv
                .write()
                .unwrap()
                .push([ctx.gen as f64, ctx.best_f]);
            task.gen.store(ctx.gen, Ordering::Relaxed);
            let time = (Instant::now() - start_time).as_secs();
            task.time.store(time, Ordering::Relaxed);
        })
        .solve(four_bar::syn::Task::new(
            &config.target,
            720,
            None,
            config.mode,
        ))
        .result()
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Synthesis {
    config: UiConfig,
    tasks: Vec<Task>,
    csv_open: bool,
    conv_open: bool,
    target_name: String,
    targets: HashMap<String, Vec<[f64; 2]>>,
}

#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(default)]
struct UiConfig {
    syn: SynConfig,
    curve_str: Arc<RwLock<String>>,
}

impl PartialEq for UiConfig {
    fn eq(&self, other: &Self) -> bool {
        self.syn == other.syn && *self.curve_str.read().unwrap() == *other.curve_str.read().unwrap()
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
enum Method {
    De,
    Fa,
    Pso,
    Rga,
    Tlbo,
}

impl Default for Method {
    fn default() -> Self {
        Self::De
    }
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

    const fn abbreviation(&self) -> &'static str {
        match self {
            Method::De => "DE",
            Method::Fa => "FA",
            Method::Pso => "PSO",
            Method::Rga => "RGA",
            Method::Tlbo => "TLBO",
        }
    }

    const fn url(&self) -> &'static str {
        match self {
            Method::De => "https://en.wikipedia.org/wiki/Differential_evolution",
            Method::Fa => "https://en.wikipedia.org/wiki/Firefly_algorithm",
            Method::Pso => "https://en.wikipedia.org/wiki/Particle_swarm_optimization",
            Method::Rga => "https://en.wikipedia.org/wiki/Genetic_algorithm",
            Method::Tlbo => "https://doi.org/10.1016/j.cad.2010.12.015",
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
struct SynConfig {
    method: Method,
    gen: u64,
    pop: usize,
    mode: Mode,
    #[serde(skip)]
    target: Vec<[f64; 2]>,
}

impl Default for SynConfig {
    fn default() -> Self {
        Self {
            method: Method::default(),
            gen: 50,
            pop: 400,
            mode: Mode::Close,
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
        serialize_with = "crate::atomic::serialize_u64",
        deserialize_with = "crate::atomic::deserialize_u64"
    )]
    gen: Arc<AtomicU64>,
    total_gen: u64,
    #[serde(
        serialize_with = "crate::atomic::serialize_u64",
        deserialize_with = "crate::atomic::deserialize_u64"
    )]
    time: Arc<AtomicU64>,
    conv: Arc<RwLock<Vec<[f64; 2]>>>,
}

impl Synthesis {
    pub fn show(&mut self, ui: &mut Ui, linkage: &mut Linkages) {
        ui.horizontal(|ui| {
            ui.heading("Synthesis");
            reset_button(ui, &mut self.config);
        });
        ui.group(|ui| {
            let method = &mut self.config.syn.method;
            ui.horizontal_wrapped(|ui| {
                for m in [
                    Method::De,
                    Method::Fa,
                    Method::Pso,
                    Method::Rga,
                    Method::Tlbo,
                ] {
                    let abb = m.abbreviation();
                    let name = m.name();
                    ui.selectable_value(method, m, abb).on_hover_text(name);
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.hyperlink_to(method.name(), method.url())
                    .on_hover_text(format!("More about {method}"));
            });
        });
        unit(ui, "Generation: ", &mut self.config.syn.gen, 1);
        unit(ui, "Population: ", &mut self.config.syn.pop, 1);
        let mut error = "";
        if !self.config.curve_str.read().unwrap().is_empty() {
            if let Some(curve) = parse_curve(&self.config.curve_str.read().unwrap()) {
                self.config.syn.target = curve;
            } else {
                error = "The provided curve is invalid.";
            }
        } else {
            error = "The target curve is empty.";
        }
        if ui.button("✏ Target Curve Editor").clicked() {
            self.csv_open = !self.csv_open;
        }
        if !error.is_empty() {
            ui.colored_label(Color32::RED, error);
            self.config.syn.target = Default::default();
        }
        ui.group(|ui| {
            ui.heading("Local Computation");
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
                    } else if ui.small_button("🗑").clicked() {
                        keep = false;
                    }
                    ui.label(format!("{}s", task.time.load(Ordering::Relaxed)));
                    let pb = task.gen.load(Ordering::Relaxed) as f32 / task.total_gen as f32;
                    ui.add(ProgressBar::new(pb).show_percentage().animate(start));
                });
                keep
            });
            ui.horizontal(|ui| {
                #[cfg(target_arch = "wasm32")]
                let _ = ui.add_enabled(false, Button::new("▶ Start"));
                #[cfg(not(target_arch = "wasm32"))]
                if ui
                    .add_enabled(error.is_empty(), Button::new("▶ Start"))
                    .clicked()
                {
                    self.native_syn(linkage.queue());
                }
                ui.add(ProgressBar::new(0.).show_percentage());
            });
        });
        ui.group(|ui| {
            ui.heading("Projects");
            ui.label("Results from the coupler trajectories.");
            if linkage.select_projects(ui) {
                self.with_current_project(ui, linkage);
            }
        });
        self.convergence_plot(ui);
        self.target_curve_editor(ui);
    }

    fn with_current_project(&self, ui: &mut Ui, linkage: &Linkages) {
        let curve = linkage.current_curve();
        let target = &self.config.syn.target;
        if !curve.is_empty() {
            let c = curve::cusp(&curve, self.config.syn.mode.is_open());
            ui.label(format!("Cusps of current curve: {}", c));
            let c = curve::crunode(&curve);
            ui.label(format!("Crunodes of current curve: {}", c));
        }
        if !target.is_empty() {
            let c = curve::cusp(target, self.config.syn.mode.is_target_open());
            ui.label(format!("Cusps of target curve: {}", c));
            let c = curve::crunode(target);
            ui.label(format!("Crunodes of target curve: {}", c));
        }
        if !target.is_empty() && !curve.is_empty() {
            let geo_err = curve::geo_err(target, &curve);
            ui.label(format!("Target mean error: {:.06}", geo_err))
                .on_hover_text(ERR_DES);
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
                            let name = format!("Best Fitness {}", i + 1);
                            ui.line(plot::Line::new(as_values(&*conv)).fill(-1.5).name(&name));
                            ui.points(plot::Points::new(as_values(&*conv)).name(&name).stems(0.));
                        }
                    });
            });
    }

    fn target_curve_editor(&mut self, ui: &mut Ui) {
        Window::new("✏ Target Curve Editor")
            .open(&mut self.csv_open)
            .show(ui.ctx(), |ui| {
                ui.label("Support CSV or RON array only.");
                let curve_str = self.config.curve_str.clone();
                ui.horizontal(|ui| {
                    if ui.button("🖴 Open Curves").clicked() {
                        let curve_csv = curve_str.clone();
                        io::open_csv_single(move |_, s| *curve_csv.write().unwrap() = s);
                    }
                    if ui.button("🗑 Clear").clicked() {
                        curve_str.write().unwrap().clear();
                    }
                });
                let mode = &mut self.config.syn.mode;
                ui.radio_value(mode, Mode::Close, "Close path matching");
                ui.radio_value(mode, Mode::Partial, "Close path match open path");
                ui.radio_value(mode, Mode::Open, "Open path matching");
                if !self.targets.is_empty() {
                    ui.label("Saved targets (local):");
                }
                self.targets.retain(|name, curve| {
                    ui.horizontal(|ui| {
                        if ui.button(name).clicked() {
                            *curve_str.write().unwrap() = dump_csv(curve).unwrap();
                        }
                        if ui.button("💾 Export CSV").clicked() {
                            io::save_csv_ask(curve);
                        }
                        !ui.button("🗑").clicked()
                    })
                    .inner
                });
                ui.label("Past curve data here:");
                ui.horizontal(|ui| {
                    if ui.button("✏ To CSV").clicked() {
                        write_curve_str(&curve_str, |c| dump_csv(&c).unwrap());
                    }
                    if ui.button("✏ To tuple array").clicked() {
                        write_curve_str(&curve_str, ron_pretty);
                    }
                    if ui.button("✏ To nested array").clicked() {
                        write_curve_str(&curve_str, |c| {
                            let c = c.into_iter().map(Vec::from).collect::<Vec<_>>();
                            ron_pretty(c)
                        });
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("💾 Update (local)").clicked() {
                        if let Some(curve) = parse_curve(&curve_str.read().unwrap()) {
                            self.targets.insert(self.target_name.clone(), curve);
                        }
                    }
                    ui.text_edit_singleline(&mut self.target_name);
                });
                ScrollArea::both().show(ui, |ui| {
                    let mut s = curve_str.write().unwrap();
                    let w = TextEdit::multiline(&mut *s)
                        .code_editor()
                        .desired_width(f32::INFINITY);
                    ui.add(w);
                });
            });
    }

    pub fn plot(&self, ui: &mut plot::PlotUi) {
        if !self.config.syn.target.is_empty() {
            let line = plot::Line::new(as_values(&self.config.syn.target))
                .name("Synthesis target")
                .style(plot::LineStyle::dashed_loose())
                .width(3.);
            ui.line(line);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_syn(&mut self, queue: super::project::Queue) {
        use four_bar::mh::{methods::*, rayon::spawn};
        let config = self.config.syn.clone();
        let task = Task {
            total_gen: config.gen,
            start: Arc::new(AtomicBool::new(true)),
            ..Task::default()
        };
        self.tasks.push(task.clone());
        spawn(move || {
            let four_bar = match config.method {
                Method::De => solve(&task, config, De::default()),
                Method::Fa => solve(&task, config, Fa::default()),
                Method::Pso => solve(&task, config, Pso::<f64>::default()),
                Method::Rga => solve(&task, config, Rga::<f64>::default()),
                Method::Tlbo => solve(&task, config, Tlbo::default()),
            };
            queue.push(None, four_bar);
            task.start.store(false, Ordering::Relaxed);
        });
    }
}
