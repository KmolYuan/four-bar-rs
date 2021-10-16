use csv::{Error, Reader};
use eframe::egui::*;
use four_bar::{synthesis::synthesis, FourBar};
use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::spawn,
};

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
    gen: u64,
    pop: usize,
    curve_csv: String,
    error: bool,
}

impl Default for Synthesis {
    fn default() -> Self {
        Self {
            started: Default::default(),
            progress: Default::default(),
            gen: 40,
            pop: 200,
            curve_csv: String::new(),
            error: false,
        }
    }
}

impl Synthesis {
    pub(crate) fn update(&mut self, ui: &mut Ui, four_bar: Arc<Mutex<FourBar>>) {
        ui.group(|ui| {
            ui.heading("Synthesis");
            parameter!("Generation: ", self.gen, ui);
            parameter!("Population: ", self.pop, ui);
            CollapsingHeader::new("Curve Input (CSV)")
                .default_open(true)
                .show(ui, |ui| {
                    ui.text_edit_multiline(&mut self.curve_csv);
                });
            if self.error {
                Label::new("The provided comma-separated value is empty or invalid.")
                    .text_color(Color32::RED)
                    .ui(ui);
            }
            ui.horizontal(|ui| {
                let started = self.started.load(Ordering::Relaxed);
                if started {
                    if ui.small_button("⏹").on_hover_text("Stop").clicked() {
                        self.started.store(false, Ordering::Relaxed);
                    }
                } else if ui.small_button("▶").on_hover_text("Start").clicked() {
                    if self.curve_csv.is_empty() {
                        self.error = true;
                    } else if let Ok(curve) = read_csv(&self.curve_csv) {
                        self.error = false;
                        self.start_syn(curve, four_bar);
                    } else {
                        self.error = true;
                    }
                }
                ProgressBar::new(self.progress.load(Ordering::Relaxed) as f32 / self.gen as f32)
                    .show_percentage()
                    .animate(started)
                    .ui(ui);
            });
        });
    }

    fn start_syn(&mut self, curve: Vec<[f64; 2]>, four_bar: Arc<Mutex<FourBar>>) {
        self.started.store(true, Ordering::Relaxed);
        let gen = self.gen;
        let pop = self.pop;
        let started = self.started.clone();
        let progress = self.progress.clone();
        spawn(move || {
            let (ans, _) = synthesis(&curve, gen, pop, |r| {
                progress.store(r.gen, Ordering::Relaxed);
                started.load(Ordering::Relaxed)
            });
            started.store(false, Ordering::Relaxed);
            *four_bar.lock().unwrap() = ans;
        });
    }
}
