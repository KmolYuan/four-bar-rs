use super::{
    project::Projects,
    remote::Remote,
    widgets::{switch_same, unit},
    IoCtx,
};
use crate::{as_values::as_values, dump_csv, parse_csv};
use eframe::egui::{
    plot::{Legend, Line, LineStyle, Plot, PlotUi, Points},
    reset_button, Button, Color32, ProgressBar, ScrollArea, Ui, Window,
};
use four_bar::tests::CRUNODE;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, RwLock,
};

const EXAMPLE_LIST: &[(&str, &[[f64; 2]])] = &[("crunode", CRUNODE)];

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Synthesis {
    #[serde(skip)]
    started: Arc<AtomicBool>,
    #[serde(skip)]
    progress: Arc<AtomicU64>,
    #[serde(
        serialize_with = "crate::atomic::serialize_u64",
        deserialize_with = "crate::atomic::deserialize_u64"
    )]
    timer: Arc<AtomicU64>,
    config: SynConfig,
    csv_open: bool,
    #[serde(skip)]
    curve: Arc<Vec<[f64; 2]>>,
    conv_open: bool,
    conv: Vec<Arc<RwLock<Vec<[f64; 2]>>>>,
    remote: Remote,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
struct SynConfig {
    gen: u64,
    pop: usize,
    open: bool,
    curve_csv: Arc<RwLock<String>>,
}

impl Default for SynConfig {
    fn default() -> Self {
        Self {
            gen: 40,
            pop: 300,
            open: false,
            curve_csv: Default::default(),
        }
    }
}

impl PartialEq for SynConfig {
    fn eq(&self, other: &Self) -> bool {
        self.gen == other.gen
            && self.pop == other.pop
            && self.open == other.open
            && *self.curve_csv.read().unwrap() == *other.curve_csv.read().unwrap()
    }
}

impl Synthesis {
    pub(crate) fn show(&mut self, ui: &mut Ui, ctx: &IoCtx, projects: &mut Projects) {
        ui.heading("Synthesis");
        reset_button(ui, &mut self.config);
        self.convergence_plot(ui);
        self.target_curve_editor(ui);
        ui.add(unit("Generation: ", &mut self.config.gen, 1));
        ui.add(unit("Population: ", &mut self.config.pop, 1));
        ui.checkbox(&mut self.config.open, "Is open curve");
        let mut error = "";
        if !self.config.curve_csv.read().unwrap().is_empty() {
            if let Ok(curve) = parse_csv(&self.config.curve_csv.read().unwrap()) {
                self.curve = Arc::new(curve);
            } else {
                error = "The provided curve is invalid.";
            }
        } else {
            error = "The target curve is empty.";
        }
        if !error.is_empty() {
            ui.colored_label(Color32::RED, error);
            self.curve = Default::default();
        }
        ui.horizontal(|ui| {
            let started = self.started.load(Ordering::Relaxed);
            if started {
                if ui.small_button("⏹").on_hover_text("Stop").clicked() {
                    self.started.store(false, Ordering::Relaxed);
                }
            } else if ui
                .add_enabled(error.is_empty(), Button::new("▶").small())
                .on_hover_text("Start")
                .clicked()
            {
                if self.remote.is_login() {
                    // TODO: Connect to server
                    let _ = projects;
                    IoCtx::alert("Not yet prepared!");
                } else {
                    self.native_syn(projects);
                }
            }
            let pb = self.progress.load(Ordering::Relaxed) as f32 / self.config.gen as f32;
            ui.add(ProgressBar::new(pb).show_percentage().animate(started));
        });
        ui.horizontal(|ui| {
            switch_same(ui, "✏", "Edit target curve", &mut self.csv_open);
            switch_same(ui, "ℹ", "Convergence window", &mut self.conv_open);
            if ui
                .small_button("🗑")
                .on_hover_text("Clear the past convergence report")
                .clicked()
                && !self.conv.is_empty()
            {
                self.conv.drain(..self.conv.len() - 1);
            }
            let time = self.timer.load(Ordering::Relaxed);
            ui.label(format!("Time passed: {}s", time));
        });
        ui.group(|ui| self.remote.show(ui, ctx));
    }

    pub(crate) fn convergence_plot(&mut self, ui: &mut Ui) {
        Window::new("Convergence Plot")
            .open(&mut self.conv_open)
            .show(ui.ctx(), |ui| {
                Plot::new("conv_canvas")
                    .legend(Legend::default())
                    .allow_drag(false)
                    .allow_zoom(false)
                    .show(ui, |ui| {
                        for (i, values) in self.conv.iter().enumerate() {
                            let values = values.read().unwrap();
                            let name = format!("Best Fitness {}", i + 1);
                            ui.line(Line::new(as_values(&values)).fill(-1.5).name(&name));
                            ui.points(Points::new(as_values(&values)).name(&name).stems(0.));
                        }
                    });
            });
    }

    pub(crate) fn target_curve_editor(&mut self, ui: &mut Ui) {
        Window::new("Target Curve Editor")
            .open(&mut self.csv_open)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("🖴 Open CSV").clicked() {
                        let curve_csv = self.config.curve_csv.clone();
                        let done = move |_, s| *curve_csv.write().unwrap() = s;
                        IoCtx::open("Delimiter-Separated Values", &["csv", "txt"], done);
                    }
                    if ui.button("🗑 Clear").clicked() {
                        *self.config.curve_csv.write().unwrap() = String::new();
                    }
                });
                ui.label("Example targets:");
                ui.horizontal(|ui| {
                    for &(name, path) in EXAMPLE_LIST {
                        if ui.button(name).clicked() {
                            *self.config.curve_csv.write().unwrap() = dump_csv(path).unwrap();
                        }
                    }
                });
                ui.label("Past CSV data here:");
                ScrollArea::vertical().max_height(450.).show(ui, |ui| {
                    ui.code_editor(&mut *self.config.curve_csv.write().unwrap());
                });
            });
    }

    pub(crate) fn plot(&self, ui: &mut PlotUi) {
        if !self.curve.is_empty() {
            let line = Line::new(as_values(&self.curve))
                .name("Synthesis target")
                .style(LineStyle::dashed_loose())
                .width(3.);
            ui.line(line);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn native_syn(&mut self, _projects: &mut Projects) {
        IoCtx::alert("Please login first!");
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_syn(&mut self, projects: &mut Projects) {
        let proj = projects.push_lazy();
        self.started.store(true, Ordering::Relaxed);
        self.timer.store(0, Ordering::Relaxed);
        let config = self.config.clone();
        let started = self.started.clone();
        let progress = self.progress.clone();
        let timer = self.timer.clone();
        let curve = self.curve.clone();
        let conv = Arc::new(RwLock::new(Vec::new()));
        self.conv.push(conv.clone());
        std::thread::spawn(move || {
            use four_bar::synthesis::{
                mh::{De, Solver},
                Planar,
            };
            let start_time = std::time::Instant::now();
            let four_bar = Solver::build(De::default())
                .pop_num(config.pop)
                .task(|ctx| ctx.gen == config.gen || !started.load(Ordering::Relaxed))
                .callback(|ctx| {
                    conv.write().unwrap().push([ctx.gen as f64, ctx.best_f]);
                    progress.store(ctx.gen, Ordering::Relaxed);
                    timer.store(
                        (std::time::Instant::now() - start_time).as_secs(),
                        Ordering::Relaxed,
                    );
                })
                .solve(Planar::new(&curve, 720, 90, config.open))
                .result();
            proj.set_four_bar(four_bar);
            started.store(false, Ordering::Relaxed);
        });
    }
}
