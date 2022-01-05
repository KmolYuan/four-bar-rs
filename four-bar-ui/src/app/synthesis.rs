use super::{remote::Remote, IoCtx};
use crate::{
    as_values::as_values,
    csv_io::{dump_csv, parse_csv},
};
use eframe::egui::{
    emath::Numeric,
    plot::{Legend, Line, LineStyle, Plot, PlotUi, Points},
    reset_button, Color32, DragValue, ProgressBar, Ui, Window,
};
use four_bar::{tests::CRUNODE, FourBar};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, RwLock,
};

fn parameter<'a>(label: &'static str, attr: &'a mut impl Numeric) -> DragValue<'a> {
    DragValue::new(attr).prefix(label).speed(1)
}

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
    #[serde(skip)]
    curve: Arc<Vec<[f64; 2]>>,
    conv_open: bool,
    conv: Vec<Arc<RwLock<Vec<[f64; 2]>>>>,
    remote: Remote,
}

#[derive(Deserialize, Serialize)]
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
            pop: 200,
            open: false,
            curve_csv: Arc::new(RwLock::new(dump_csv(CRUNODE).unwrap())),
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
    pub(crate) fn ui(&mut self, ui: &mut Ui, ctx: &IoCtx, four_bar: Arc<RwLock<FourBar>>) {
        ui.heading("Synthesis");
        reset_button(ui, &mut self.config);
        let iter = self.conv.iter().enumerate();
        Window::new("Convergence Plot")
            .open(&mut self.conv_open)
            .show(ui.ctx(), |ui| {
                Plot::new("conv_canvas")
                    .legend(Legend::default())
                    .allow_drag(false)
                    .allow_zoom(false)
                    .show(ui, |ui| {
                        for (i, values) in iter {
                            let values = values.read().unwrap();
                            let name = format!("Best Fitness {}", i + 1);
                            ui.line(Line::new(as_values(&values)).fill(-1.5).name(&name));
                            ui.points(Points::new(as_values(&values)).name(&name).stems(0.));
                        }
                    });
            });
        ui.add(parameter("Generation: ", &mut self.config.gen));
        ui.add(parameter("Population: ", &mut self.config.pop));
        ui.checkbox(&mut self.config.open, "Is open curve");
        if ui.button("Open CSV").clicked() {
            let curve_csv = self.config.curve_csv.clone();
            ctx.open("Delimiter-Separated Values", &["csv", "txt"], move |s| {
                *curve_csv.write().unwrap() = s;
            });
        }
        ui.collapsing("Curve Input (CSV)", |ui| {
            ui.text_edit_multiline(&mut *self.config.curve_csv.write().unwrap())
        });
        if !self.config.curve_csv.read().unwrap().is_empty() {
            if let Ok(curve) = parse_csv(&self.config.curve_csv.read().unwrap()) {
                self.curve = Arc::new(curve);
            } else {
                const TEXT: &str = "The provided curve is invalid.\nUses latest valid curve.";
                ui.colored_label(Color32::RED, TEXT);
            }
        }
        ui.horizontal(|ui| {
            let started = self.started.load(Ordering::Relaxed);
            if started {
                if ui.small_button("⏹").on_hover_text("Stop").clicked() {
                    self.started.store(false, Ordering::Relaxed);
                }
            } else if ui.small_button("▶").on_hover_text("Start").clicked()
                && !self.curve.is_empty()
            {
                if self.remote.is_login() {
                    // TODO: Connect to server
                    let _ = four_bar;
                    IoCtx::alert("Not yet prepared!");
                } else {
                    self.native_syn(four_bar);
                }
            }
            let pb = self.progress.load(Ordering::Relaxed) as f32 / self.config.gen as f32;
            ui.add(ProgressBar::new(pb).show_percentage().animate(started));
        });
        ui.horizontal(|ui| {
            if ui
                .small_button("ℹ")
                .on_hover_text("Convergence window")
                .clicked()
            {
                self.conv_open = !self.conv_open;
            }
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
        ui.group(|ui| self.remote.ui(ui, ctx));
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
    fn native_syn(&mut self, _four_bar: Arc<RwLock<FourBar>>) {
        IoCtx::alert("Please login first!");
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_syn(&mut self, four_bar: Arc<RwLock<FourBar>>) {
        self.started.store(true, Ordering::Relaxed);
        self.timer.store(0, Ordering::Relaxed);
        let gen = self.config.gen;
        let pop = self.config.pop;
        let open = self.config.open;
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
            *four_bar.write().unwrap() = Solver::build(De::default())
                .pop_num(pop)
                .task(|ctx| ctx.gen == gen || !started.load(Ordering::Relaxed))
                .callback(|ctx| {
                    conv.write().unwrap().push([ctx.gen as f64, ctx.best_f]);
                    progress.store(ctx.gen, Ordering::Relaxed);
                    timer.store(
                        (std::time::Instant::now() - start_time).as_secs(),
                        Ordering::Relaxed,
                    );
                })
                .solve(Planar::new(&curve, 720, 360, open))
                .result();
            started.store(false, Ordering::Relaxed);
        });
    }
}
