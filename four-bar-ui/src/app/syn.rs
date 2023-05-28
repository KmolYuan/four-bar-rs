use super::{link::Linkages, widgets::*};
use crate::{io, syn_cmd, syn_cmd::Target::*};
use eframe::egui::*;
use four_bar::*;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[inline]
fn ron_pretty<S: ?Sized + Serialize>(s: &S) -> String {
    ron::ser::to_string_pretty(s, Default::default()).unwrap()
}

#[derive(Deserialize, Serialize, Clone)]
struct Task {
    time: std::time::Duration,
    conv: Vec<f64>,
}

#[derive(Default)]
enum Cache {
    #[default]
    Empty,
    Curve(io::Curve),
    Cb(io::Cb),
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Synthesis {
    method: syn_cmd::SynMethod,
    cfg: syn_cmd::SynConfig,
    target: syn_cmd::Target,
    tasks: Vec<Task>,
    #[serde(skip)]
    queue: Rc<RefCell<Cache>>,
    #[serde(skip)]
    task_queue: Vec<Arc<mutex::RwLock<(f32, Task)>>>,
    #[serde(skip)]
    conv_open: bool,
    #[serde(skip)]
    from_plot_open: bool,
}

impl Synthesis {
    pub(crate) fn show(&mut self, ui: &mut Ui, lnk: &mut Linkages) {
        ui.horizontal(|ui| {
            ui.heading("Synthesis");
            reset_button(ui, &mut self.cfg);
        });
        ui.group(|ui| self.opt_setting(ui));
        check_on(ui, "Random seed", &mut self.cfg.seed, 0, any_i);
        nonzero_i(ui, "Generation: ", &mut self.cfg.gen, 1);
        nonzero_i(ui, "Population: ", &mut self.cfg.pop, 1);
        ui.horizontal(|ui| {
            ui.label("Mode: ");
            for (mode, name) in [
                (syn::Mode::Closed, "Closed"),
                (syn::Mode::Open, "Open"),
                (syn::Mode::Partial, "Partial"),
            ] {
                ui.selectable_value(&mut self.cfg.mode, mode, name);
            }
        });
        ui.separator();
        match self.target {
            P(_, _) => ui.heading("Planar Target Curve"),
            S(_, _) => ui.heading("Spherical Target Curve"),
        };
        match std::mem::replace(&mut *self.queue.borrow_mut(), Cache::Empty) {
            Cache::Curve(curve) => self.target.set_curve(curve),
            Cache::Cb(cb) => io::alert(self.target.set_cb(cb), |_| ()),
            Cache::Empty => (),
        }
        toggle_btn(ui, &mut self.from_plot_open, "🖊 Add from canvas")
            .on_hover_text("Click canvas to add target point drictly!");
        ui.horizontal(|ui| {
            if ui.button("🖊 Add from").clicked() {
                match (lnk.projs.current_curve(), &mut self.target) {
                    (io::Curve::P(c), P(t, _)) => *t = c,
                    (io::Curve::S(c), S(t, _)) => *t = c,
                    (io::Curve::P(c), t @ S(_, _)) => *t = P(c, Default::default()),
                    (io::Curve::S(c), t @ P(_, _)) => *t = S(c, Default::default()),
                }
            }
            lnk.projs.select(ui, false);
        });
        ui.horizontal(|ui| {
            if ui.button("🖴 Load").clicked() {
                let queue = self.queue.clone();
                io::open_csv_single(move |_, c| *queue.borrow_mut() = Cache::Curve(c));
            }
            if ui.button("💾 Save CSV").clicked() {
                match &self.target {
                    P(t, _) => io::save_csv_ask(t),
                    S(t, _) => io::save_csv_ask(t),
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("🗐 Copy CSV").clicked() {
                let text = match &self.target {
                    P(t, _) => csv::dump_csv(t).unwrap(),
                    S(t, _) => csv::dump_csv(t).unwrap(),
                };
                ui.output_mut(|s| s.copied_text = text);
            }
            if ui.button("🗐 Copy Array of Tuple").clicked() {
                let text = match &self.target {
                    P(t, _) => ron_pretty(t),
                    S(t, _) => ron_pretty(t),
                };
                ui.output_mut(|s| s.copied_text = text);
            }
            if ui.button("🗐 Copy Array of Array").clicked() {
                macro_rules! vec_nest {
                    ($iter:ident) => {
                        $iter.iter().map(|c| c.to_vec()).collect::<Vec<_>>()
                    };
                }
                let text = match &self.target {
                    P(t, _) => ron_pretty(&vec_nest!(t)),
                    S(t, _) => ron_pretty(&vec_nest!(t)),
                };
                ui.output_mut(|s| s.copied_text = text);
            }
        });
        ui.group(|ui| match &mut self.target {
            P(t, _) => table(ui, t),
            S(t, _) => table(ui, t),
        });
        ui.separator();
        ui.heading("Codebook");
        ui.label("Use pre-searched dataset to increase the speed.");
        ui.label(format!(
            "Number of data: {}",
            match &self.target {
                P(_, cb) => cb.len(),
                S(_, cb) => cb.len(),
            }
        ));
        ui.label("Run \"four-bar cb\" in command line window to generate codebook file.");
        ui.horizontal(|ui| {
            if ui.button("🖴 Load").clicked() {
                let queue = self.queue.clone();
                io::open_cb(move |cb| *queue.borrow_mut() = Cache::Cb(cb));
            }
            if ui.button("🗑 Clear").clicked() {
                match &mut self.target {
                    P(_, cb) => cb.clear(),
                    S(_, cb) => cb.clear(),
                }
            }
        });
        ui.separator();
        ui.heading("Optimization");
        toggle_btn(ui, &mut self.conv_open, "📉 Convergence Plot");
        self.tasks.retain(|task| {
            ui.horizontal(|ui| {
                if small_btn(ui, "🗑", "Delete") {
                    return false;
                }
                if small_btn(ui, "💾", "Save history plot") {
                    io::save_history_ask(&task.conv, "history.svg");
                }
                ui.label(format!("{:.4?}", task.time));
                ui.colored_label(Color32::GREEN, "Finished");
                true
            })
            .inner
        });
        self.task_queue.retain(|task| {
            let (pg, task) = &mut *task.write();
            ui.horizontal(|ui| {
                if small_btn(ui, "⏹", "Stop") {
                    *pg = 1.;
                }
                ui.label(format!("{:.4?}", task.time));
                ui.add(ProgressBar::new(*pg).show_percentage().animate(true));
            });
            // FIXME: Use `drain_filter`
            if *pg == 1. {
                self.tasks.push(task.clone());
                false
            } else {
                true
            }
        });
        #[cfg(target_arch = "wasm32")]
        ui.colored_label(Color32::RED, "Web version freezes UI when solving starts!");
        ui.horizontal(|ui| {
            let enabled = self.target.has_target();
            if ui.add_enabled(enabled, Button::new("▶ Start")).clicked() {
                self.start_syn(lnk);
            }
            ui.add(ProgressBar::new(0.).show_percentage());
        });
        self.convergence_plot(ui);
    }

    fn opt_setting(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for &(name, abbr, f) in syn_cmd::SynMethod::LIST {
                let c = self.method.abbr() == abbr;
                if ui.selectable_label(c, abbr).on_hover_text(name).clicked() && !c {
                    self.method = f();
                }
            }
        });
        let m = &mut self.method;
        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to(m.name(), m.link())
                .on_hover_text(format!("More about {}", m.name()));
        });
        macro_rules! param {
            ($s:ident, $($name:ident),+) => {{$(
                percent(ui, concat![stringify!($name), ": "], &mut $s.$name);
            )+}};
        }
        use syn_cmd::SynMethod::*;
        match m {
            De(s) => {
                ui.horizontal_wrapped(|ui| {
                    use mh::de::Strategy::*;
                    for (i, strategy) in [S1, S2, S3, S4, S5, S6, S7, S8, S9, S10]
                        .into_iter()
                        .enumerate()
                    {
                        ui.selectable_value(&mut s.strategy, strategy, format!("S{i}"));
                    }
                });
                param!(s, f, cross);
            }
            Fa(s) => param!(s, alpha, beta_min, gamma),
            Pso(s) => param!(s, cognition, social, velocity),
            Rga(s) => param!(s, cross, mutate, win, delta),
            Tlbo(_) => (),
        }
    }

    fn convergence_plot(&mut self, ui: &mut Ui) {
        Window::new("📉 Convergence Plot")
            .open(&mut self.conv_open)
            .show(ui.ctx(), |ui| {
                plot::Plot::new("plot_conv")
                    .legend(Default::default())
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_scroll(false)
                    .show(ui, |ui| {
                        for (i, task) in self.tasks.iter().enumerate() {
                            let pts1 = plot::PlotPoints::from_ys_f64(&task.conv);
                            let pts2 = plot::PlotPoints::from_ys_f64(&task.conv);
                            let name = format!("Task {}", i + 1);
                            ui.line(plot::Line::new(pts1).fill(-1.5).name(&name));
                            ui.points(plot::Points::new(pts2).name(name).stems(0.));
                        }
                    });
            });
    }

    pub(crate) fn plot(&mut self, ui: &mut plot::PlotUi, lnk: &Linkages) {
        if self.target.has_target() {
            const NAME: &str = "Synthesis target";
            let target = match &self.target {
                P(t, _) => t.clone(),
                S(t, _) => t.iter().map(|[x, y, _]| [*x, *y]).collect(),
            };
            let line = plot::Line::new(target.clone())
                .name(NAME)
                .style(plot::LineStyle::dashed_loose())
                .width(3.);
            ui.line(line);
            let points = plot::Points::new(target)
                .name(NAME)
                .filled(false)
                .radius(5.);
            ui.points(points);
        }
        if !self.from_plot_open || !ui.plot_clicked() {
            return;
        }
        // Add target curve from canvas
        let p = ui.pointer_coordinate().unwrap();
        match &mut self.target {
            P(t, _) => t.push([p.x, p.y]),
            S(t, _) => {
                let f = || {
                    let [sx, sy, sz, r] = lnk.projs.current_sphere()?;
                    let dx = p.x - sx;
                    let dy = p.y - sy;
                    (dx.hypot(dy) <= r).then_some([p.x, p.y, r * r - dx * dx - dy * dy + sz])
                };
                if let Some(c) = f() {
                    t.push(c);
                } else {
                    let p = plot::Points::new([p.x, p.y])
                        .shape(plot::MarkerShape::Cross)
                        .color(Color32::RED)
                        .radius(30.);
                    ui.points(p);
                }
            }
        }
    }

    fn start_syn(&mut self, lnk: &Linkages) {
        #[cfg(target_arch = "wasm32")]
        use instant::Instant;
        #[cfg(not(target_arch = "wasm32"))]
        use std::time::Instant;
        let task = Task {
            time: std::time::Duration::from_secs(0),
            conv: Vec::new(),
        };
        let task = Arc::new(mutex::RwLock::new((0., task)));
        self.task_queue.push(task.clone());
        let total_gen = self.cfg.gen;
        let t0 = Instant::now();
        let s = syn_cmd::Solver::new(
            self.method.clone(),
            self.target.clone(),
            self.cfg.clone(),
            move |best_f, gen| {
                let (pg, task) = &mut *task.write();
                *pg = gen as f32 / total_gen as f32;
                task.conv.push(best_f);
                task.time = t0.elapsed();
            },
        );
        let queue = lnk.projs.queue();
        let f = move || io::alert(s.solve(), |fb| queue.push(None, fb));
        #[cfg(not(target_arch = "wasm32"))]
        four_bar::mh::rayon::spawn(f);
        #[cfg(target_arch = "wasm32")]
        f(); // Block
    }
}
