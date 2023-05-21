use super::{io, widgets::*};
use crate::syn_cmd::SynCmd;
use eframe::egui::*;
use four_bar::*;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[inline]
fn ron_pretty<S: ?Sized + Serialize>(s: &S) -> String {
    ron::ser::to_string_pretty(s, Default::default()).unwrap()
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Synthesis {
    cfg_method: SynCmd,
    cfg: SynConfig,
    target: Target,
    tasks: Vec<Task>,
    #[serde(skip)]
    queue: Rc<RefCell<Cache>>,
    #[serde(skip)]
    task_queue: Vec<Arc<mutex::RwLock<(u64, Task)>>>,
    #[serde(skip)]
    conv_open: bool,
    #[serde(skip)]
    from_plot_open: bool,
}

#[derive(Default)]
enum Cache {
    #[default]
    Empty,
    Curve(io::Curve),
    Cb(io::Cb),
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
struct SynConfig {
    gen: u64,
    pop: usize,
    mode: syn::Mode,
}

impl Default for SynConfig {
    fn default() -> Self {
        Self { gen: 50, pop: 200, mode: syn::Mode::Closed }
    }
}

#[derive(Deserialize, Serialize, Clone)]
enum Target {
    P(Vec<[f64; 2]>, #[serde(skip)] cb::FbCodebook),
    S(Vec<[f64; 3]>, #[serde(skip)] cb::SFbCodebook),
}

impl Default for Target {
    fn default() -> Self {
        Self::P(Vec::new(), Default::default())
    }
}

impl Target {
    fn set_curve(&mut self, curve: io::Curve) {
        match (curve, self) {
            (io::Curve::P(c), Self::P(t, _)) => *t = c,
            (io::Curve::S(c), Self::S(t, _)) => *t = c,
            (io::Curve::P(c), t @ Self::S(_, _)) => *t = Self::P(c, Default::default()),
            (io::Curve::S(c), t @ Self::P(_, _)) => *t = Self::S(c, Default::default()),
        }
    }

    fn set_cb(&mut self, cb: io::Cb) -> Result<(), mh::ndarray::ShapeError> {
        match (cb, self) {
            (io::Cb::P(c), Self::P(_, t)) => t.merge_inplace(&c)?,
            (io::Cb::S(c), Self::S(_, t)) => t.merge_inplace(&c)?,
            (io::Cb::P(c), t @ Self::S(_, _)) => *t = Self::P(Vec::new(), c),
            (io::Cb::S(c), t @ Self::P(_, _)) => *t = Self::S(Vec::new(), c),
        }
        Ok(())
    }

    fn has_target(&self) -> bool {
        match self {
            Self::P(t, _) => !t.is_empty(),
            Self::S(t, _) => !t.is_empty(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
struct Task {
    gen: u64,
    time_spent: u64,
    conv: Vec<f64>,
}

impl Synthesis {
    pub(crate) fn show(&mut self, ui: &mut Ui, lnk: &mut super::link::Linkages) {
        ui.horizontal(|ui| {
            ui.heading("Synthesis");
            reset_button(ui, &mut self.cfg);
        });
        ui.group(|ui| self.opt_setting(ui));
        nonzero_i(ui, "Generation: ", &mut self.cfg.gen, 1);
        nonzero_i(ui, "Population: ", &mut self.cfg.pop, 1);
        ui.separator();
        match self.target {
            Target::P(_, _) => ui.heading("Planar Target Curve"),
            Target::S(_, _) => ui.heading("Spherical Target Curve"),
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
                    (io::Curve::P(c), Target::P(t, _)) => *t = c,
                    (io::Curve::S(c), Target::S(t, _)) => *t = c,
                    (io::Curve::P(c), t @ Target::S(_, _)) => *t = Target::P(c, Default::default()),
                    (io::Curve::S(c), t @ Target::P(_, _)) => *t = Target::S(c, Default::default()),
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
                    Target::P(t, _) => io::save_csv_ask(t),
                    Target::S(t, _) => io::save_csv_ask(t),
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("🗐 Copy CSV").clicked() {
                let text = match &self.target {
                    Target::P(t, _) => csv::dump_csv(t).unwrap(),
                    Target::S(t, _) => csv::dump_csv(t).unwrap(),
                };
                ui.output_mut(|s| s.copied_text = text);
            }
            if ui.button("🗐 Copy Array of Tuple").clicked() {
                let text = match &self.target {
                    Target::P(t, _) => ron_pretty(t),
                    Target::S(t, _) => ron_pretty(t),
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
                    Target::P(t, _) => ron_pretty(&vec_nest!(t)),
                    Target::S(t, _) => ron_pretty(&vec_nest!(t)),
                };
                ui.output_mut(|s| s.copied_text = text);
            }
        });
        ui.group(|ui| match &mut self.target {
            Target::P(t, _) => table(ui, t),
            Target::S(t, _) => table(ui, t),
        });
        ui.separator();
        ui.heading("Codebook");
        ui.label("Use pre-searched dataset to increase the speed.");
        ui.label(format!(
            "Number of data: {}",
            match &self.target {
                Target::P(_, cb) => cb.len(),
                Target::S(_, cb) => cb.len(),
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
                    Target::P(_, cb) => cb.clear(),
                    Target::S(_, cb) => cb.clear(),
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
                let t = std::time::Duration::from_secs(task.time_spent);
                ui.label(format!("{t:?}"));
                ui.add(ProgressBar::new(1.).show_percentage());
                true
            })
            .inner
        });
        self.task_queue.retain(|task| {
            ui.horizontal(|ui| {
                let (gen, task) = &mut *task.write();
                if *gen != task.gen {
                    if small_btn(ui, "⏹", "Stop") {
                        task.gen = *gen;
                    }
                } else {
                    if small_btn(ui, "🗑", "Delete") {
                        return false;
                    }
                    if small_btn(ui, "💾", "Save history plot") {
                        io::save_history_ask(&task.conv, "history.svg");
                    }
                }
                let t = std::time::Duration::from_secs(task.time_spent);
                ui.label(format!("{t:?}"));
                let pb = ProgressBar::new(*gen as f32 / task.gen as f32)
                    .show_percentage()
                    .animate(*gen != task.gen);
                ui.add(pb);
                true
            })
            .inner
        });
        // FIXME: Use `drain_filter`
        self.task_queue.retain(|task| {
            let (gen, task) = &*task.read();
            if *gen >= task.gen {
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
                self.start_syn(lnk.projs.queue());
            }
            ui.add(ProgressBar::new(0.).show_percentage());
        });
        self.convergence_plot(ui);
    }

    fn opt_setting(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for &(name, abbr, f) in SynCmd::LIST {
                let c = self.cfg_method.abbr() == abbr;
                if ui.selectable_label(c, abbr).on_hover_text(name).clicked() && !c {
                    self.cfg_method = f();
                }
            }
        });
        let m = &mut self.cfg_method;
        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to(m.name(), m.link())
                .on_hover_text(format!("More about {}", m.name()));
        });
        macro_rules! param {
            ($s:ident, $($name:ident),+) => {{$(
                percent(ui, concat![stringify!($name), ": "], &mut $s.$name);
            )+}};
        }
        match m {
            SynCmd::De(s) => {
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
            SynCmd::Fa(s) => param!(s, alpha, beta_min, gamma),
            SynCmd::Pso(s) => param!(s, cognition, social, velocity),
            SynCmd::Rga(s) => param!(s, cross, mutate, win, delta),
            SynCmd::Tlbo(_) => (),
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

    pub(crate) fn plot(&mut self, ui: &mut plot::PlotUi) {
        if self.from_plot_open && ui.plot_clicked() {
            // Add target curve from canvas
            #[allow(unused_variables)]
            let plot::PlotPoint { x, y } = ui.pointer_coordinate().unwrap();
            // TODO: Interactives
        }
        if self.target.has_target() {
            const NAME: &str = "Synthesis target";
            let target = match &self.target {
                Target::P(t, _) => t.clone(),
                Target::S(t, _) => t.iter().map(|[x, y, _]| [*x, *y]).collect(),
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
    }

    fn start_syn(&mut self, queue: super::proj::Queue) {
        #[cfg(not(target_arch = "wasm32"))]
        use four_bar::mh::rayon::spawn;
        let target = self.target.clone();
        let method = self.cfg_method.clone();
        let pop = self.cfg.pop;
        let mode = self.cfg.mode;
        let task = Task { gen: self.cfg.gen, time_spent: 0, conv: Vec::new() };
        let task = Arc::new(mutex::RwLock::new((0, task)));
        self.task_queue.push(task.clone());
        let f = move || {
            let fb = match method {
                SynCmd::De(s) => SynSolver::new(s, target, pop, mode, task).solve(),
                SynCmd::Fa(s) => SynSolver::new(s, target, pop, mode, task).solve(),
                SynCmd::Pso(s) => SynSolver::new(s, target, pop, mode, task).solve(),
                SynCmd::Rga(s) => SynSolver::new(s, target, pop, mode, task).solve(),
                SynCmd::Tlbo(s) => SynSolver::new(s, target, pop, mode, task).solve(),
            };
            queue.push(None, fb);
        };
        #[cfg(not(target_arch = "wasm32"))]
        spawn(f);
        #[cfg(target_arch = "wasm32")]
        f(); // Block
    }
}

struct SynSolver<S: mh::Setting> {
    setting: S,
    target: Target,
    pop: usize,
    mode: syn::Mode,
    task: Arc<mutex::RwLock<(u64, Task)>>,
}

impl<S: mh::Setting> SynSolver<S> {
    fn new(
        setting: S,
        target: Target,
        pop: usize,
        mode: syn::Mode,
        task: Arc<mutex::RwLock<(u64, Task)>>,
    ) -> Self {
        Self { setting, target, pop, mode, task }
    }

    fn solve(self) -> io::Fb {
        #[cfg(target_arch = "wasm32")]
        use instant::Instant;
        #[cfg(not(target_arch = "wasm32"))]
        use std::time::Instant;
        let t0 = Instant::now();
        let Self { setting, target, pop, mode, task } = self;
        macro_rules! impl_solve {
            ($target:ident, $cb:ident, $fb:ident, $syn:ident) => {{
                let mut s =
                    four_bar::mh::Solver::build(setting, syn::$syn::from_curve(&$target, mode));
                if let Some(candi) = matches!(mode, syn::Mode::Closed | syn::Mode::Open)
                    .then(|| $cb.fetch_raw(&$target, mode.is_target_open(), pop))
                    .filter(|candi| !candi.is_empty())
                {
                    s = s.pop_num(candi.len());
                    let fitness = candi
                        .iter()
                        .map(|(f, fb)| mh::Product::new(*f, fb.denormalize()))
                        .collect();
                    let pool = candi.into_iter().map(|(_, fb)| fb.buf).collect::<Vec<_>>();
                    s = s.pool_and_fitness(mh::ndarray::arr2(&pool), fitness);
                } else {
                    s = s.pop_num(pop);
                }
                let fb = s
                    .task(|ctx| ctx.gen >= task.read().1.gen)
                    .callback(|ctx| {
                        let (gen, task) = &mut *task.write();
                        task.conv.push(ctx.best_f.fitness());
                        *gen = ctx.gen;
                        task.time_spent = t0.elapsed().as_secs();
                    })
                    .solve()
                    .unwrap()
                    .into_result();
                io::Fb::$fb(fb)
            }};
        }
        match target {
            Target::P(target, cb) => impl_solve!(target, cb, Fb, FbSyn),
            Target::S(target, cb) => impl_solve!(target, cb, SFb, SFbSyn),
        }
    }
}
