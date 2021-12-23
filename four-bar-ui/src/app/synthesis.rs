use super::{remote::Remote, Atomic, IoCtx};
use crate::{
    as_values::as_values,
    csv_io::{dump_csv, parse_csv},
};
use eframe::egui::{
    plot::{Legend, Line, Plot, Points},
    Color32, DragValue, Label, ProgressBar, Ui, Widget, Window,
};
use four_bar::{tests::CRUNODE, FourBar};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
#[cfg(not(target_arch = "wasm32"))]
use {four_bar::synthesis::synthesis, std::time::Instant};

macro_rules! parameter {
    ($label:literal, $attr:expr, $ui:ident) => {
        DragValue::new(&mut $attr)
            .prefix($label)
            .clamp_range(0..=5000)
            .speed(1)
            .ui($ui);
    };
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
    curve_csv: String,
    pub(crate) curve: Arc<Vec<[f64; 2]>>,
    conv_open: bool,
    conv: Vec<Arc<RwLock<Vec<[f64; 2]>>>>,
    remote: Remote,
}

impl Default for Synthesis {
    fn default() -> Self {
        Self {
            started: Default::default(),
            progress: Default::default(),
            #[cfg(not(target_arch = "wasm32"))]
            timer: Default::default(),
            gen: 40,
            pop: 200,
            curve_csv: dump_csv(CRUNODE).unwrap(),
            curve: Arc::new(CRUNODE.to_vec()),
            conv_open: false,
            conv: Default::default(),
            #[cfg(not(target_arch = "wasm32"))]
            remote: Remote::with_address("http://localhost:8080/"),
            #[cfg(target_arch = "wasm32")]
            remote: Default::default(),
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
                let mut plot = Plot::new("conv_canvas");
                for (i, values) in iter {
                    let values = values.read().unwrap();
                    let name = format!("Best Fitness {}", i + 1);
                    plot = plot
                        .line(Line::new(as_values(&values)).fill(-1.5).name(&name))
                        .points(Points::new(as_values(&values)).name(&name).stems(0.));
                }
                plot.legend(Legend::default())
                    .allow_drag(false)
                    .allow_zoom(false)
                    .ui(ui);
            });
        parameter!("Generation: ", self.gen, ui);
        parameter!("Population: ", self.pop, ui);
        if ui.button("Open CSV").clicked() {
            #[cfg(target_arch = "wasm32")]
            let _ = ctx.open(&["csv", "txt"]);
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(s) = ctx.open("Delimiter-Separated Values", &["csv", "txt"]) {
                self.curve_csv = s;
            }
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(s) = ctx.open_result() {
            self.curve_csv = s;
        }
        ui.collapsing("Curve Input (CSV)", |ui| {
            ui.text_edit_multiline(&mut self.curve_csv)
        });
        if !self.curve_csv.is_empty() {
            if let Ok(curve) = parse_csv(&self.curve_csv) {
                self.curve = Arc::new(curve);
            } else {
                Label::new("The provided curve is invalid.\nUses latest valid curve.")
                    .text_color(Color32::RED)
                    .ui(ui);
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
                #[cfg(not(target_arch = "wasm32"))]
                self.native_syn(four_bar);
                #[cfg(target_arch = "wasm32")]
                {
                    // TODO: Connect to server
                    let _ = four_bar;
                    IoCtx::alert("Not yet prepared!");
                }
            }
            ProgressBar::new(self.progress.load() as f32 / self.gen as f32)
                .show_percentage()
                .animate(started)
                .ui(ui);
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
            let started_inner = started.clone();
            *four_bar.write().unwrap() = synthesis(
                &curve,
                pop,
                move |ctx| {
                    conv.write().unwrap().push([ctx.gen as f64, ctx.best_f]);
                    progress.store(ctx.gen);
                    let time = Instant::now() - start_time;
                    timer.store(time.as_secs());
                    ctx.gen == gen || !started_inner.load()
                },
                |_| (),
            )
            .result();
            started.store(false);
        });
    }
}
