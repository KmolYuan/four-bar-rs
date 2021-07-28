use crate::switch_button;
use eframe::egui::*;
use four_bar::synthesis::synthesis;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::thread::spawn;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct Synthesis {
    started: bool,
    progress: Arc<AtomicU32>,
}

impl Default for Synthesis {
    fn default() -> Self {
        Self {
            started: false,
            progress: Default::default(),
        }
    }
}

impl Synthesis {
    pub fn update(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Synthesis");
            ui.horizontal(|ui| {
                let started = self.started;
                switch_button!(ui, self.started, "⏹", "Stop", "▶", "Start");
                if !started && self.started {
                    let progress = self.progress.clone();
                    spawn(move || {
                        synthesis(YU2, 40, 200, |r| {
                            progress.store(r.gen, Ordering::Relaxed);
                            false
                        })
                    });
                }
                if self.started {
                    // TODO: Progress bar here!
                    ui.label(self.progress.load(Ordering::Relaxed).to_string());
                    ui.ctx().request_repaint();
                }
            });
        });
    }
}

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
