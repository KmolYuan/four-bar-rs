use super::{
    project::Queue,
    remote::Remote,
    widgets::{switch_same, unit},
    IoCtx,
};
use crate::{as_values::as_values, dump_csv, parse_csv};
use eframe::egui::{
    plot::{Legend, Line, LineStyle, Plot, PlotUi, Points},
    reset_button, Button, Color32, ProgressBar, ScrollArea, Ui, Window,
};
use four_bar::tests::{CRUNODE, OPEN_CURVE2};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, RwLock,
};

const EXAMPLE_LIST: &[(&str, &[[f64; 2]])] = &[("crunode", CRUNODE), ("open curve 2", OPEN_CURVE2)];

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Synthesis {
    config: UiConfig,
    #[serde(skip)]
    curve: Vec<[f64; 2]>,
    tasks: Vec<Task>,
    csv_open: bool,
    conv_open: bool,
    remote: Remote,
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
}

impl Default for SynConfig {
    fn default() -> Self {
        Self {
            gen: 50,
            pop: 300,
            open: false,
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
    pub(crate) fn show(&mut self, ui: &mut Ui, ctx: &IoCtx, queue: Queue) {
        ui.heading("Synthesis");
        reset_button(ui, &mut self.config);
        self.convergence_plot(ui);
        self.target_curve_editor(ui);
        ui.add(unit("Generation: ", &mut self.config.syn.gen, 1));
        ui.add(unit("Population: ", &mut self.config.syn.pop, 1));
        let mut error = "";
        if !self.config.curve_csv.read().unwrap().is_empty() {
            if let Ok(curve) = parse_csv(&self.config.curve_csv.read().unwrap()) {
                self.curve = curve;
            } else {
                error = "The provided curve is invalid.";
            }
        } else {
            error = "The target curve is empty.";
        }
        ui.horizontal(|ui| {
            switch_same(ui, "✏", "Edit target curve", &mut self.csv_open);
            if !error.is_empty() {
                ui.colored_label(Color32::RED, error);
                self.curve = Default::default();
            }
        });
        ui.group(|ui| {
            ui.heading("Local Computation");
            switch_same(ui, "ℹ", "Convergence window", &mut self.conv_open);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(error.is_empty(), Button::new("▶ Start"))
                    .clicked()
                {
                    self.native_syn(queue);
                }
                ui.add(ProgressBar::new(0.).show_percentage());
            });
            self.tasks.retain(|task| {
                ui.horizontal(|ui| {
                    let mut keep = true;
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
                    keep
                })
                .inner
            });
        });
        ui.group(|ui| self.remote.show(ui, ctx));
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
                        let done = move |_, s| *curve_csv.write().unwrap() = s;
                        IoCtx::open("Delimiter-Separated Values", &["csv", "txt"], done);
                    }
                    if ui.button("🗑 Clear").clicked() {
                        *self.config.curve_csv.write().unwrap() = String::new();
                    }
                    ui.checkbox(&mut self.config.syn.open, "Is open curve");
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
    fn native_syn(&mut self, _queue: Queue) {
        IoCtx::alert("Local computation is not supported!");
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_syn(&mut self, queue: Queue) {
        use four_bar::synthesis::{
            mh::{utility::thread::spawn, De, Solver},
            Planar,
        };
        let curve = self.curve.clone();
        let SynConfig { pop, gen, open } = self.config.syn;
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
                .solve(Planar::new(&curve, 720, 90, open))
                .result();
            queue.push(None, four_bar);
            task.start.store(false, Ordering::Relaxed);
        });
    }
}
