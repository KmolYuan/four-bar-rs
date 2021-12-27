use super::{remote::Remote, Atomic, IoCtx};
use crate::{
    as_values::as_values,
    csv_io::{dump_csv, parse_csv},
};
use eframe::egui::{
    emath::Numeric,
    plot::{Legend, Line, Plot, Points},
    Color32, DragValue, Label, ProgressBar, Ui, Window,
};
#[cfg(not(target_arch = "wasm32"))]
use four_bar::synthesis::{
    mh::{De, Solver},
    Planar,
};
use four_bar::{tests::CRUNODE, FourBar};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

fn parameter<'a>(label: &'a str, attr: &'a mut impl Numeric) -> DragValue<'a> {
    DragValue::new(attr)
        .prefix(label)
        .clamp_range(0..=10000)
        .speed(1)
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Synthesis {
    #[serde(skip)]
    started: Atomic<bool>,
    #[serde(skip)]
    progress: Atomic<u64>,
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    timer: Atomic<u64>,
    gen: u64,
    pop: usize,
    curve_csv: Arc<RwLock<String>>,
    pub(crate) curve: Arc<Vec<[f64; 2]>>,
    conv_open: bool,
    conv: Vec<Arc<RwLock<Vec<[f64; 2]>>>>,
    remote: Remote,
}

impl Default for Synthesis {
    fn default() -> Self {
        Self {
            started: Atomic::from(false),
            progress: Atomic::from(0),
            #[cfg(not(target_arch = "wasm32"))]
            timer: Atomic::from(0),
            gen: 40,
            pop: 200,
            curve_csv: Arc::new(RwLock::new(dump_csv(CRUNODE).unwrap())),
            curve: Arc::new(CRUNODE.to_vec()),
            conv_open: false,
            conv: Vec::new(),
            remote: Remote::default(),
        }
    }
}

impl Synthesis {
    pub(crate) fn ui(&mut self, ui: &mut Ui, ctx: &IoCtx, four_bar: Arc<RwLock<FourBar>>) {
        ui.heading("Synthesis");
        let iter = self.conv.iter().enumerate();
        Window::new("Convergence Plot")
            .open(&mut self.conv_open)
            .show(ui.ctx(), |ui| {
                let mut plot = Plot::new("conv_canvas")
                    .legend(Legend::default())
                    .allow_drag(false)
                    .allow_zoom(false);
                for (i, values) in iter {
                    let values = values.read().unwrap();
                    let name = format!("Best Fitness {}", i + 1);
                    plot = plot
                        .line(Line::new(as_values(&values)).fill(-1.5).name(&name))
                        .points(Points::new(as_values(&values)).name(&name).stems(0.));
                }
                ui.add(plot);
            });
        ui.add(parameter("Generation: ", &mut self.gen));
        ui.add(parameter("Population: ", &mut self.pop));
        if ui.button("Open CSV").clicked() {
            let curve_csv = self.curve_csv.clone();
            ctx.open("Delimiter-Separated Values", &["csv", "txt"], move |s| {
                *curve_csv.write().unwrap() = s;
            });
        }
        ui.collapsing("Curve Input (CSV)", |ui| {
            ui.text_edit_multiline(&mut *self.curve_csv.write().unwrap())
        });
        if !self.curve_csv.read().unwrap().is_empty() {
            if let Ok(curve) = parse_csv(&self.curve_csv.read().unwrap()) {
                self.curve = Arc::new(curve);
            } else {
                let label = Label::new("The provided curve is invalid.\nUses latest valid curve.")
                    .text_color(Color32::RED);
                ui.add(label);
            }
        }
        ui.horizontal(|ui| {
            let started = self.started.load();
            if started {
                if ui.small_button("⏹").on_hover_text("Stop").clicked() {
                    self.started.store(false);
                }
            } else if ui.small_button("▶").on_hover_text("Start").clicked()
                && !self.curve.is_empty()
            {
                if self.remote.is_login() {
                    // TODO: Connect to server
                    let _ = four_bar;
                    IoCtx::alert("Not yet prepared!");
                } else {
                    #[cfg(target_arch = "wasm32")]
                    let _ = IoCtx::alert("Please login first!");
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = self.native_syn(four_bar);
                }
            }
            let pb = ProgressBar::new(self.progress.load() as f32 / self.gen as f32)
                .show_percentage()
                .animate(started);
            ui.add(pb);
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
            #[cfg(not(target_arch = "wasm32"))]
            ui.label(format!("Time passed: {}s", self.timer.load()));
        });
        ui.group(|ui| self.remote.ui(ui, ctx));
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_syn(&mut self, four_bar: Arc<RwLock<FourBar>>) {
        self.started = Atomic::from(true);
        self.timer.store(0);
        let gen = self.gen;
        let pop = self.pop;
        let started = self.started.clone();
        let progress = self.progress.clone();
        let timer = self.timer.clone();
        let curve = self.curve.clone();
        let conv = Arc::new(RwLock::new(Vec::new()));
        self.conv.push(conv.clone());
        std::thread::spawn(move || {
            let start_time = Instant::now();
            *four_bar.write().unwrap() = Solver::build(De::default())
                .pop_num(pop)
                .task(|ctx| ctx.gen == gen || !started.load())
                .callback(|ctx| {
                    conv.write().unwrap().push([ctx.gen as f64, ctx.best_f]);
                    progress.store(ctx.gen);
                    timer.store((Instant::now() - start_time).as_secs());
                })
                .solve(Planar::new(&curve, 720, 360))
                .result();
            started.store(false);
        });
    }
}
