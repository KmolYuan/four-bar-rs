use crate::as_values::as_values;
use csv::{Error, Reader};
use eframe::egui::*;
use four_bar::{synthesis::synthesis, FourBar};
use rayon::spawn;
use rfd::FileDialog;
use std::{
    fs::read_to_string,
    io::Cursor,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};

const CRUNODE: &str = include_str!("assets/crunode.csv");

macro_rules! parameter {
    ($label:literal, $attr:expr, $ui:ident) => {
        DragValue::new(&mut $attr)
            .prefix($label)
            .clamp_range(0..=5000)
            .speed(1)
            .ui($ui);
    };
}

fn read_csv(s: &str) -> Result<Vec<[f64; 2]>, Error> {
    Reader::from_reader(Cursor::new(s))
        .deserialize::<[f64; 2]>()
        .collect()
}

#[cfg_attr(
    feature = "persistence",
    derive(serde::Deserialize, serde::Serialize),
    serde(default)
)]
pub(crate) struct Synthesis {
    started: Arc<AtomicBool>,
    progress: Arc<AtomicU64>,
    timer: Arc<AtomicU64>,
    gen: u64,
    pop: usize,
    curve_csv: String,
    pub(crate) curve: Arc<Vec<[f64; 2]>>,
    conv_open: bool,
    conv: Arc<Mutex<Vec<[f64; 2]>>>,
    error: String,
}

impl Default for Synthesis {
    fn default() -> Self {
        Self {
            started: Default::default(),
            progress: Default::default(),
            timer: Default::default(),
            gen: 40,
            pop: 200,
            curve_csv: CRUNODE.to_string(),
            curve: Default::default(),
            conv_open: false,
            conv: Default::default(),
            error: String::new(),
        }
    }
}

impl Synthesis {
    pub(crate) fn update(&mut self, ui: &mut Ui, four_bar: Arc<Mutex<FourBar>>) {
        ui.heading("Synthesis");
        let conv = self.conv.clone();
        Window::new("Convergence Plot")
            .open(&mut self.conv_open)
            .show(ui.ctx(), |ui| {
                let values = conv.lock().unwrap();
                plot::Plot::new("conv_canvas")
                    .line(
                        plot::Line::new(as_values(&values))
                            .fill(-1.5)
                            .name("Best Fitness"),
                    )
                    .points(
                        plot::Points::new(as_values(&values))
                            .name("Best Fitness")
                            .stems(0.)
                            .filled(true),
                    )
                    .legend(plot::Legend::default())
                    .allow_drag(false)
                    .allow_zoom(false)
                    .ui(ui);
            });
        parameter!("Generation: ", self.gen, ui);
        parameter!("Population: ", self.pop, ui);
        if ui.button("Open CSV").clicked() {
            if let Some(file) = FileDialog::new()
                .add_filter("Delimiter-Separated Values", &["txt", "csv"])
                .pick_file()
            {
                if let Ok(curve_csv) = read_to_string(file) {
                    self.curve_csv = curve_csv;
                } else {
                    self.error = "Invalid text file.".to_string();
                }
            }
        }
        CollapsingHeader::new("Curve Input (CSV)")
            .default_open(true)
            .show(ui, |ui| {
                if ui.text_edit_multiline(&mut self.curve_csv).changed() {
                    self.error.clear();
                }
            });
        if !self.curve_csv.is_empty() {
            if let Ok(curve) = read_csv(&self.curve_csv) {
                self.curve = Arc::new(curve);
            } else {
                self.error = "The provided curve is invalid.".to_string();
            }
        }
        if !self.error.is_empty() {
            Label::new(&self.error).text_color(Color32::RED).ui(ui);
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
                self.start_syn(four_bar);
            }
            ProgressBar::new(self.progress.load(Ordering::Relaxed) as f32 / self.gen as f32)
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
            ui.label(format!(
                "Time passed: {}s",
                self.timer.load(Ordering::Relaxed)
            ));
        });
    }

    fn start_syn(&mut self, four_bar: Arc<Mutex<FourBar>>) {
        self.started.store(true, Ordering::Relaxed);
        self.timer.store(0, Ordering::Relaxed);
        let gen = self.gen;
        let pop = self.pop;
        let started = self.started.clone();
        let progress = self.progress.clone();
        let timer = self.timer.clone();
        let curve = self.curve.clone();
        let conv = self.conv.clone();
        spawn(move || {
            let start_time = Instant::now();
            let (ans, _) = synthesis(&curve, gen, pop, |r| {
                conv.lock().unwrap().push([r.gen as f64, r.best_f]);
                progress.store(r.gen, Ordering::Relaxed);
                let time = Instant::now() - start_time;
                timer.store(time.as_secs(), Ordering::Relaxed);
                started.load(Ordering::Relaxed)
            });
            *four_bar.lock().unwrap() = ans;
            started.store(false, Ordering::Relaxed);
        });
    }
}
