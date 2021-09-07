use eframe::egui::*;
use four_bar::synthesis::synthesis;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};
use std::thread::spawn;

macro_rules! parameter {
    ($label:literal, $attr:expr, $ui:ident) => {
        DragValue::new(&mut $attr)
            .prefix($label)
            .clamp_range(0..=5000)
            .speed(1)
            .ui($ui);
    };
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct Synthesis {
    started: Arc<AtomicBool>,
    progress: Arc<AtomicU32>,
    gen: u32,
    pop: usize,
}

impl Default for Synthesis {
    fn default() -> Self {
        Self {
            started: Arc::new(AtomicBool::new(false)),
            progress: Default::default(),
            gen: 40,
            pop: 200,
        }
    }
}

impl Synthesis {
    pub fn update(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Synthesis");
            parameter!("Generation: ", self.gen, ui);
            parameter!("Population: ", self.pop, ui);
            ui.horizontal(|ui| {
                let started = self.started.load(Ordering::Relaxed);
                if started {
                    if ui.small_button("⏹").on_hover_text("Stop").clicked() {
                        self.started.store(false, Ordering::Relaxed);
                    }
                } else if ui.small_button("▶").on_hover_text("Start").clicked() {
                    self.started.store(true, Ordering::Relaxed);
                    let gen = self.gen;
                    let pop = self.pop;
                    let started = self.started.clone();
                    let progress = self.progress.clone();
                    spawn(move || {
                        let ans = synthesis(YU2, gen, pop, |r| {
                            progress.store(r.gen, Ordering::Relaxed);
                            started.load(Ordering::Relaxed)
                        });
                        started.store(false, Ordering::Relaxed);
                        ans
                    });
                }
                ProgressBar::new(self.progress.load(Ordering::Relaxed) as f32 / self.gen as f32)
                    .show_percentage()
                    .animate(started)
                    .ui(ui);
            });
        });
    }
}

// FIXME: Remove this test case
const YU2: &[[f64; 2]] = &[
    [-24., 40.],
    [-30., 41.],
    [-34., 40.],
    [-38., 36.],
    [-36., 30.],
    [-28., 29.],
    [-21., 31.],
    [-17., 32.],
    [-8., 34.],
    [3., 37.],
    [10., 41.],
    [17., 41.],
    [26., 39.],
    [28., 33.],
    [29., 26.],
    [26., 23.],
    [17., 23.],
    [11., 24.],
    [6., 27.],
    [0., 31.],
];
