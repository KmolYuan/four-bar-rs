use super::{linkages::Linkages, project::Queue, remote::Remote, widgets::unit, IoCtx};
use crate::{as_values::as_values, dump_csv, parse_csv};
use eframe::egui::{
    plot::{Legend, Line, LineStyle, Plot, PlotUi, Points},
    reset_button, Button, Color32, ProgressBar, ScrollArea, TextEdit, Ui, Window,
};
use four_bar::curve;
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

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Synthesis {
    config: UiConfig,
    tasks: Vec<Task>,
    csv_open: bool,
    conv_open: bool,
    remote: Remote,
    target_name: String,
    targets: HashMap<String, Vec<[f64; 2]>>,
}

#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(default)]
struct UiConfig {
    syn: SynConfig,
    curve_csv: Arc<RwLock<String>>,
}

impl PartialEq for UiConfig {
    fn eq(&self, other: &Self) -> bool {
        self.syn == other.syn && *self.curve_csv.read().unwrap() == *other.curve_csv.read().unwrap()
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
struct SynConfig {
    gen: u64,
    pop: usize,
    open: bool,
    #[serde(skip)]
    target: Vec<[f64; 2]>,
}

impl Default for SynConfig {
    fn default() -> Self {
        Self {
            gen: 50,
            pop: 400,
            open: false,
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
    pub fn show(&mut self, ui: &mut Ui, ctx: &IoCtx, linkage: &mut Linkages) {
        ui.heading("Synthesis");
        reset_button(ui, &mut self.config);
        self.convergence_plot(ui);
        self.target_curve_editor(ui);
        ui.add(unit("Generation: ", &mut self.config.syn.gen, 1));
        ui.add(unit("Population: ", &mut self.config.syn.pop, 1));
        let mut error = "";
        if !self.config.curve_csv.read().unwrap().is_empty() {
            let curve_csv = self.config.curve_csv.read().unwrap();
            if let Ok(curve) = parse_csv(&curve_csv) {
                self.config.syn.target = curve;
            } else if let Ok(curve) = ron::from_str::<Vec<Vec<f64>>>(&curve_csv) {
                self.config.syn.target = curve.into_iter().map(|c| [c[0], c[1]]).collect();
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
            if ui.button("🗠 Convergence Plot").clicked() {
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
                if ui
                    .add_enabled(error.is_empty(), Button::new("▶ Start"))
                    .clicked()
                {
                    self.native_syn(linkage.queue());
                }
                ui.add(ProgressBar::new(0.).show_percentage());
            });
        });
        ui.group(|ui| self.remote.show(ui, ctx));
        ui.group(|ui| {
            ui.heading("Projects");
            ui.label("Results from the coupler trajectories.");
            if linkage.select_projects(ui) {
                self.with_current_project(ui, linkage);
            }
        });
    }

    fn with_current_project(&self, ui: &mut Ui, linkage: &Linkages) {
        let curve = linkage.current_curve();
        if !curve.is_empty() {
            let c = curve::crunode(&curve);
            ui.label(format!("Crunodes of current curve: {}", c));
        }
        if !self.config.syn.target.is_empty() {
            let c = curve::crunode(&self.config.syn.target);
            ui.label(format!("Crunodes of target curve: {}", c));
        }
        if !self.config.syn.target.is_empty() && !curve.is_empty() {
            let geo_err = curve::geo_err(&self.config.syn.target, &curve);
            ui.label(format!("Target mean error: {:.06}", geo_err))
                .on_hover_text(ERR_DES);
        }
    }

    fn convergence_plot(&mut self, ui: &mut Ui) {
        Window::new("Convergence Plot")
            .open(&mut self.conv_open)
            .show(ui.ctx(), |ui| {
                Plot::new("conv_canvas")
                    .legend(Legend::default())
                    .allow_drag(false)
                    .allow_zoom(false)
                    .show(ui, |ui| {
                        for (i, task) in self.tasks.iter().enumerate() {
                            let conv = task.conv.read().unwrap();
                            let name = format!("Best Fitness {}", i + 1);
                            ui.line(Line::new(as_values(&*conv)).fill(-1.5).name(&name));
                            ui.points(Points::new(as_values(&*conv)).name(&name).stems(0.));
                        }
                    });
            });
    }

    fn target_curve_editor(&mut self, ui: &mut Ui) {
        Window::new("Target Curve Editor")
            .open(&mut self.csv_open)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("🖴 Open CSV").clicked() {
                        let curve_csv = self.config.curve_csv.clone();
                        IoCtx::open_csv_single(move |_, s| {
                            curve_csv.write().unwrap().clone_from(&s)
                        });
                    }
                    if ui.button("🗑 Clear").clicked() {
                        self.config.curve_csv.write().unwrap().clear();
                    }
                    ui.checkbox(&mut self.config.syn.open, "Is open curve");
                });
                if !self.targets.is_empty() {
                    ui.label("Saved targets (local):");
                }
                self.targets.retain(|name, curve| {
                    ui.horizontal(|ui| {
                        if ui.button(name).clicked() {
                            self.config
                                .curve_csv
                                .write()
                                .unwrap()
                                .clone_from(&dump_csv(curve).unwrap());
                        }
                        if ui.button("💾 Export CSV").clicked() {
                            IoCtx::save_csv_ask(curve);
                        }
                        !ui.button("🗑").clicked()
                    })
                    .inner
                });
                ui.label("Past CSV data here:");
                ui.horizontal(|ui| {
                    if ui.button("💾 Update (local)").clicked() {
                        if let Ok(curve) = parse_csv(&self.config.curve_csv.read().unwrap()) {
                            self.targets.insert(self.target_name.clone(), curve);
                        }
                    }
                    ui.text_edit_singleline(&mut self.target_name);
                });
                ScrollArea::both().auto_shrink([true; 2]).show(ui, |ui| {
                    let mut s = self.config.curve_csv.write().unwrap();
                    let w = TextEdit::multiline(&mut *s)
                        .code_editor()
                        .desired_width(f32::INFINITY);
                    ui.add(w);
                });
            });
    }

    pub fn plot(&self, ui: &mut PlotUi) {
        if !self.config.syn.target.is_empty() {
            let line = Line::new(as_values(&self.config.syn.target))
                .name("Synthesis target")
                .style(LineStyle::dashed_loose())
                .width(3.);
            ui.line(line);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn native_syn(&mut self, _queue: Queue) {
        IoCtx::alert("Local computation is not supported!");
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_syn(&mut self, queue: Queue) {
        use four_bar::synthesis::{
            mh::{rayon::spawn, De, Solver},
            Planar,
        };
        let SynConfig {
            pop,
            gen,
            open,
            target,
        } = self.config.syn.clone();
        let task = Task {
            total_gen: gen,
            start: Arc::new(AtomicBool::new(true)),
            ..Task::default()
        };
        self.tasks.push(task.clone());
        spawn(move || {
            let start_time = std::time::Instant::now();
            let four_bar = Solver::build(De::default())
                .pop_num(pop)
                .task(|ctx| ctx.gen == gen || !task.start.load(Ordering::Relaxed))
                .callback(|ctx| {
                    task.conv
                        .write()
                        .unwrap()
                        .push([ctx.gen as f64, ctx.best_f]);
                    task.gen.store(ctx.gen, Ordering::Relaxed);
                    let time = (std::time::Instant::now() - start_time).as_secs();
                    task.time.store(time, Ordering::Relaxed);
                })
                .solve(Planar::new(&target, 720, None, open))
                .result();
            queue.push(None, four_bar);
            task.start.store(false, Ordering::Relaxed);
        });
    }
}
