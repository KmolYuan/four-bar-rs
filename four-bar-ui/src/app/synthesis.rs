use super::io_ctx::IoCtx;
use crate::{as_values::as_values, csv_io::read_csv};
use eframe::egui::{
    plot::{Legend, Line, Plot, Points},
    Color32, DragValue, Label, ProgressBar, Ui, Widget, Window,
};
use four_bar::FourBar;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use {
    four_bar::synthesis::synthesis,
    std::{
        sync::atomic::{AtomicBool, AtomicU64, Ordering},
        time::Instant,
    },
};

const CRUNODE: &str = include_str!("../assets/crunode.csv");

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
    #[cfg(not(target_arch = "wasm32"))]
    started: Arc<AtomicBool>,
    #[cfg(target_arch = "wasm32")]
    started: Arc<Mutex<bool>>,
    #[cfg(not(target_arch = "wasm32"))]
    progress: Arc<AtomicU64>,
    #[cfg(target_arch = "wasm32")]
    progress: Arc<Mutex<u64>>,
    #[cfg(not(target_arch = "wasm32"))]
    timer: Arc<AtomicU64>,
    gen: u64,
    pop: usize,
    curve_csv: String,
    pub(crate) curve: Arc<Vec<[f64; 2]>>,
    conv_open: bool,
    conv: Vec<Arc<Mutex<Vec<[f64; 2]>>>>,
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
            curve_csv: CRUNODE.to_string(),
            curve: Default::default(),
            conv_open: false,
            conv: Default::default(),
        }
    }
}

impl Synthesis {
    pub(crate) fn ui(&mut self, ui: &mut Ui, ctx: &IoCtx, four_bar: Arc<Mutex<FourBar>>) {
        ui.heading("Synthesis");
        let iter = self.conv.iter().enumerate();
        Window::new("Convergence Plot")
            .open(&mut self.conv_open)
            .show(ui.ctx(), |ui| {
                let mut plot = Plot::new("conv_canvas");
                for (i, values) in iter {
                    let values = values.lock().unwrap();
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
            {
                ctx.open(&["txt", "csv"]);
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.curve_csv = ctx.open("Delimiter-Separated Values", &["txt", "csv"]);
            }
            #[cfg(target_arch = "wasm32")]
            if let Some(s) = ctx.open_result() {
                self.curve_csv = s;
            }
        }
        ui.collapsing("Curve Input (CSV)", |ui| {
            ui.text_edit_multiline(&mut self.curve_csv)
        });
        if !self.curve_csv.is_empty() {
            if let Ok(curve) = read_csv(&self.curve_csv) {
                self.curve = Arc::new(curve);
            } else {
                Label::new("The provided curve is invalid.\nUses latest valid curve.")
                    .text_color(Color32::RED)
                    .ui(ui);
            }
        }
        ui.horizontal(|ui| {
            #[cfg(not(target_arch = "wasm32"))]
            let started = self.started.load(Ordering::Relaxed);
            #[cfg(target_arch = "wasm32")]
            let started = false;
            if started {
                if ui.small_button("⏹").on_hover_text("Stop").clicked() {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.started.store(false, Ordering::Relaxed);
                }
            } else if ui.small_button("▶").on_hover_text("Start").clicked()
                && !self.curve.is_empty()
            {
                #[cfg(not(target_arch = "wasm32"))]
                self.start_syn(four_bar);
                #[cfg(target_arch = "wasm32")]
                // TODO: Connect to server
                let _ = four_bar;
            }
            #[cfg(not(target_arch = "wasm32"))]
            let progress = self.progress.load(Ordering::Relaxed);
            #[cfg(target_arch = "wasm32")]
            let progress = 0;
            ProgressBar::new(progress as f32 / self.gen as f32)
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
            ui.label(format!(
                "Time passed: {}s",
                self.timer.load(Ordering::Relaxed)
            ));
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_syn(&mut self, four_bar: Arc<Mutex<FourBar>>) {
        self.started = Arc::new(AtomicBool::new(true));
        self.timer.store(0, Ordering::Relaxed);
        let gen = self.gen;
        let pop = self.pop;
        let started = self.started.clone();
        let progress = self.progress.clone();
        let timer = self.timer.clone();
        let curve = self.curve.clone();
        let conv = Arc::new(Mutex::new(Vec::new()));
        self.conv.push(conv.clone());
        std::thread::spawn(move || {
            let start_time = Instant::now();
            let s = synthesis(&curve, gen, pop, |r| {
                conv.lock().unwrap().push([r.gen as f64, r.best_f]);
                progress.store(r.gen, Ordering::Relaxed);
                let time = Instant::now() - start_time;
                timer.store(time.as_secs(), Ordering::Relaxed);
                started.load(Ordering::Relaxed)
            });
            *four_bar.lock().unwrap() = s.result();
            started.store(false, Ordering::Relaxed);
        });
    }
}
