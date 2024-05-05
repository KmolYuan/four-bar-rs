use super::{link::Linkages, widgets::*};
use crate::{io, syn_cmd, syn_cmd::Target};
use eframe::egui::*;
use four_bar::{atlas, csv, mh, syn};
use serde::{Deserialize, Serialize};
use std::{
    iter::zip,
    sync::{
        atomic::{
            AtomicU32,
            Ordering::{Relaxed, SeqCst},
        },
        Arc, Mutex,
    },
};

#[inline]
fn ron_pretty<S: ?Sized + Serialize>(s: &S) -> String {
    ron::ser::to_string_pretty(s, Default::default()).unwrap()
}

#[derive(Deserialize, Serialize, Clone)]
struct Task {
    time: std::time::Duration,
    conv: Vec<f64>,
}

#[derive(Clone)]
struct TaskInProg {
    pg: Arc<AtomicU32>,
    task: Arc<Mutex<Task>>,
}

impl TaskInProg {
    fn new(task: Task) -> Self {
        Self {
            pg: Arc::new(AtomicU32::new(0f32.to_bits())),
            task: Arc::new(Mutex::new(task)),
        }
    }
}

#[derive(Default)]
enum Cache {
    #[default]
    Empty,
    Curve(io::Curve),
    Atlas(io::Atlas),
}

struct AtlasVis {
    pt: [f64; 2],
    is_open: bool,
    is_sphere: bool,
}

#[derive(Deserialize, Serialize)]
struct AtlasCfg {
    size: usize,
    harmonic: usize,
    is_open: bool,
}

impl Default for AtlasCfg {
    fn default() -> Self {
        Self { size: 10000, harmonic: 20, is_open: false }
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Synthesis {
    alg: syn_cmd::SynAlg,
    cfg: syn_cmd::SynCfg,
    atlas_cfg: AtlasCfg,
    target: io::Curve,
    tasks: Vec<Task>,
    #[serde(skip)]
    task_queue: Vec<TaskInProg>,
    #[serde(skip)]
    atlas: io::AtlasPool,
    #[serde(skip)]
    atlas_vis: Vec<AtlasVis>,
    #[serde(skip)]
    atlas_pg: Option<Arc<AtomicU32>>,
    #[serde(skip)]
    queue: Arc<mutex::Mutex<Cache>>,
    #[serde(skip)]
    conv_open: bool,
    #[serde(skip)]
    atlas_vis_open: bool,
    #[serde(skip)]
    from_plot_open: bool,
}

impl Synthesis {
    pub(crate) fn show(&mut self, ui: &mut Ui, lnk: &mut Linkages) {
        ui.horizontal(|ui| {
            ui.heading("Synthesis");
            reset_button(ui, &mut self.cfg);
        });
        ui.collapsing("Algorithm", |ui| {
            ui.group(|ui| self.opt_setting(ui));
            check_on(ui, "Random seed", &mut self.cfg.seed, any_i);
            nonzero_i(ui, "Generation: ", &mut self.cfg.gen, 1);
            nonzero_i(ui, "Population: ", &mut self.cfg.pop, 1);
            nonzero_i(ui, "Resolution: ", &mut self.cfg.res, 1);
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.cfg.use_dd, "Use distance discrepancy");
                hint(ui, "When the point number is small, the distance discrepancy may help to find the better solution.");
            });
        });
        ui.collapsing("Atlas Database", |ui| self.atlas_setting(ui));
        ui.separator();
        ui.heading("Target Curve");
        ui.horizontal(|ui| {
            ui.group(|ui| {
                if ui
                    .selectable_label(matches!(self.target, io::Curve::P(..)), "Planar")
                    .clicked()
                {
                    self.target.convert_to_planar();
                }
                if ui
                    .selectable_label(matches!(self.target, io::Curve::M(..)), "Motion")
                    .clicked()
                {
                    self.target.convert_to_motion();
                }
                if ui
                    .selectable_label(matches!(self.target, io::Curve::S(..)), "Spherical")
                    .clicked()
                {
                    self.target.convert_to_spatial();
                }
            });
        });
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
        match std::mem::replace(&mut *self.queue.lock(), Cache::Empty) {
            Cache::Curve(curve) => self.target = curve,
            Cache::Atlas(atlas) => io::alert!("Merge Atlas", self.atlas.merge_inplace(atlas)),
            Cache::Empty => (),
        }
        ui.checkbox(&mut self.cfg.on_unit, "Constrain on unit");
        ui.horizontal(|ui| {
            toggle_btn(ui, &mut self.from_plot_open, "🖊 Append Mode");
            hint(ui, "Add points by clicking the canvas");
        });
        ui.horizontal(|ui| {
            if ui.button("🖴 Load from").clicked() {
                if let Some(target) = lnk.projs.current_curve() {
                    self.target = target;
                }
            }
            lnk.projs.select(ui);
        });
        ui.horizontal(|ui| {
            if ui.button("🖴 Load from CSV").clicked() {
                let queue = self.queue.clone();
                io::open_csv_single(move |_, c| *queue.lock() = Cache::Curve(c));
            }
            if ui.button("💾 Save CSV").clicked() {
                match &self.target {
                    io::Curve::P(t) => io::save_csv_ask(t),
                    io::Curve::M(t) => io::save_csv_ask(t),
                    io::Curve::S(t) => io::save_csv_ask(t),
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("🗐 Copy CSV").clicked() {
                let text = match &self.target {
                    io::Curve::P(t) => csv::to_string(t).unwrap(),
                    io::Curve::M(t) => csv::to_string(t).unwrap(),
                    io::Curve::S(t) => csv::to_string(t).unwrap(),
                };
                ui.output_mut(|s| s.copied_text = text);
            }
            if ui.button("🗐 Copy Array of Tuple").clicked() {
                let text = match &self.target {
                    io::Curve::P(t) => ron_pretty(t),
                    io::Curve::M(t) => ron_pretty(t),
                    io::Curve::S(t) => ron_pretty(t),
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
                    io::Curve::P(t) => ron_pretty(&vec_nest!(t)),
                    io::Curve::M(t) => ron_pretty(
                        &t.iter()
                            .map(|&(c1, c2)| [c1, c2].concat())
                            .collect::<Vec<_>>(),
                    ),
                    io::Curve::S(t) => ron_pretty(&vec_nest!(t)),
                };
                ui.output_mut(|s| s.copied_text = text);
            }
        });
        match &mut self.target {
            io::Curve::P(t) => table(ui, t),
            io::Curve::M(t) => {
                // Safety: Same memory layout of `([f64; 2], [f64; 2])` and `[f64; 4]`
                table(ui, unsafe { &mut *(t as *mut _ as *mut Vec<[f64; 4]>) });
                if ui.button("Normalize vectors").clicked() {
                    for (_, v) in t {
                        let norm = v[0].hypot(v[1]);
                        v[0] /= norm;
                        v[1] /= norm;
                    }
                }
            }
            io::Curve::S(t) => table(ui, t),
        }
        ui.separator();
        ui.heading("Optimization");
        toggle_btn(ui, &mut self.conv_open, "📉 Convergence Plot");
        self.tasks.retain(|task| {
            ui.horizontal(|ui| {
                let keep = !small_btn(ui, "✖", "Delete");
                if small_btn(ui, "💾", "Save history plot") {
                    io::save_history_ask(&task.conv, "history.svg");
                }
                ui.label(format!("{:.4?}", task.time));
                ui.colored_label(Color32::GREEN, "Finished");
                keep
            })
            .inner
        });
        for i in (0..self.task_queue.len()).rev() {
            let TaskInProg { pg, task } = &self.task_queue[i];
            ui.horizontal(|ui| {
                if small_btn(ui, "⏹", "Stop") {
                    pg.store(1f32.to_bits(), SeqCst);
                }
                ui.label(format!("{:.4?}", task.lock().unwrap().time));
                ui.add(ProgressBar::new(pg_get(pg)).show_percentage().animate(true));
            });
            // Export the finished task to history
            if Arc::strong_count(task) == 1 {
                let task = self.task_queue.swap_remove(i).task;
                self.tasks
                    .push(Arc::into_inner(task).unwrap().into_inner().unwrap());
            }
        }
        #[cfg(target_arch = "wasm32")]
        ui.colored_label(Color32::RED, "Web version freezes UI when solving starts!");
        ui.horizontal(|ui| {
            let enabled = !self.target.is_empty();
            if ui.add_enabled(enabled, Button::new("▶ Start")).clicked() {
                self.start_syn(lnk);
            }
            ui.add(ProgressBar::new(0.).show_percentage());
        });
        self.atlas_vis(ui);
        self.convergence_plot(ui);
    }

    fn opt_setting(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for &(name, abbr, f) in syn_cmd::SynAlg::LIST {
                let c = self.alg.abbr() == abbr;
                if ui.selectable_label(c, abbr).on_hover_text(name).clicked() && !c {
                    self.alg = f();
                }
            }
        });
        let m = &mut self.alg;
        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to(m.name(), m.link())
                .on_hover_text(format!("More about {}", m.name()));
        });
        macro_rules! param {
            ($s:ident, $($name:ident),+) => {{$(
                percent(ui, concat![stringify!($name), ": "], &mut $s.$name);
            )+}};
        }
        use syn_cmd::SynAlg::*;
        match m {
            De(s) => {
                const LIST: [mh::de::Strategy; 10] = mh::de::Strategy::LIST;
                combo_enum(ui, "de strategy", &mut s.strategy, LIST, |e| {
                    let i = LIST.iter().position(|s| s == e).unwrap() + 1;
                    format!("S{i}")
                });
                param!(s, f, cross);
            }
            Fa(s) => param!(s, alpha, beta_min, gamma),
            Pso(s) => param!(s, cognition, social, velocity),
            Rga(s) => param!(s, cross, mutate, win, delta),
            Tlbo(_) => (),
        }
    }

    fn atlas_setting(&mut self, ui: &mut Ui) {
        ui.label("Use pre-searched dataset to increase the synthesis performance.");
        ui.horizontal(|ui| {
            if ui.button("🖴 Load").clicked() {
                let queue = self.queue.clone();
                io::open_cb(move |atlas| *queue.lock() = Cache::Atlas(atlas));
            }
            ui.group(|ui| {
                if ui.button("☁ Point Cloud Visualize").clicked() {
                    if !self.atlas_vis_open {
                        self.atlas_vis_cache();
                    } else {
                        self.atlas_vis.clear();
                        self.atlas_vis.shrink_to_fit();
                    }
                    self.atlas_vis_open = !self.atlas_vis_open;
                }
            });
        });
        ui.separator();
        ui.horizontal(|ui| {
            nonzero_i(ui, "Size: ", &mut self.atlas_cfg.size, 1);
            nonzero_i(ui, "Harmonic: ", &mut self.atlas_cfg.harmonic, 1);
            ui.checkbox(&mut self.atlas_cfg.is_open, "Is open curve");
        });
        macro_rules! impl_make_cb {
            ($atlas:ident, $atlas_ty:ident) => {
                let size = self.atlas_cfg.size;
                let cfg = atlas::Cfg::new()
                    .res(self.cfg.res)
                    .size(size)
                    .harmonic(self.atlas_cfg.harmonic)
                    .is_open(self.atlas_cfg.is_open);
                let queue = self.queue.clone();
                let pg = Arc::new(AtomicU32::new(0f32.to_bits()));
                self.atlas_pg = Some(pg.clone());
                let f = move || {
                    let atlas =
                        atlas::$atlas::make_with(cfg, |p| pg_set(&pg, p as f32 / size as f32));
                    *queue.lock() = Cache::Atlas(io::Atlas::$atlas_ty(atlas));
                };
                #[cfg(not(target_arch = "wasm32"))]
                mh::rayon::spawn(f);
                #[cfg(target_arch = "wasm32")]
                f(); // Block
            };
        }
        Grid::new("atlas_cfg").show(ui, |ui| {
            ui.label("");
            ui.label("Planar Data");
            ui.label("Spherical Data");
            ui.end_row();
            ui.label("Size");
            ui.label(self.atlas.as_fb().len().to_string());
            ui.label(self.atlas.as_sfb().len().to_string());
            ui.end_row();
            ui.label("Save");
            if ui.button("💾").clicked() {
                io::save_atlas_ask(self.atlas.as_fb());
            }
            if ui.button("💾").clicked() {
                io::save_atlas_ask(self.atlas.as_sfb());
            }
            ui.end_row();
            ui.label("Generate");
            let enabled = self.atlas_pg.is_none();
            if ui.add_enabled(enabled, Button::new("✚")).clicked() {
                impl_make_cb!(FbAtlas, P);
            }
            if ui.add_enabled(enabled, Button::new("✚")).clicked() {
                impl_make_cb!(SFbAtlas, S);
            }
            ui.end_row();
            ui.label("Clear");
            if ui.button("✖").clicked() {
                self.atlas.as_fb_mut().clear();
            }
            if ui.button("✖").clicked() {
                self.atlas.as_sfb_mut().clear();
            }
        });
        if let Some(pg) = &self.atlas_pg {
            let pg = pg_get(pg);
            ui.add(ProgressBar::new(pg).show_percentage().animate(true));
            if pg == 1. {
                self.atlas_pg = None;
            }
        }
    }

    fn convergence_plot(&mut self, ui: &mut Ui) {
        Window::new("📉 Convergence Plot")
            .open(&mut self.conv_open)
            .show(ui.ctx(), |ui| {
                static_plot("plot_conv").show(ui, |ui| {
                    let mut draw = |name: &str, task: &Task| {
                        let pts1 = egui_plot::PlotPoints::from_ys_f64(&task.conv);
                        let pts2 = egui_plot::PlotPoints::from_ys_f64(&task.conv);
                        ui.line(egui_plot::Line::new(pts1).fill(-1.5).name(name));
                        ui.points(egui_plot::Points::new(pts2).name(name).stems(0.));
                    };
                    for (i, task) in self.tasks.iter().enumerate() {
                        draw(&format!("Task {i}"), task);
                    }
                    for (i, task) in self.task_queue.iter().enumerate() {
                        draw(&format!("Queue {i}"), &task.task.lock().unwrap());
                    }
                });
            });
    }

    pub(crate) fn plot(&mut self, ui: &mut egui_plot::PlotUi, lnk: &Linkages) {
        if self.from_plot_open && ui.response().clicked() {
            // Add target curve from clicking canvas
            self.on_click_canvas(ui, lnk);
        }
        if self.target.is_empty() {
            return;
        }
        let bound = ui.plot_bounds();
        let mut draw_curve = |target: Vec<_>| {
            const NAME: &str = "Synthesis target";
            let line = egui_plot::Line::new(target.clone())
                .name(NAME)
                .color(Color32::BLUE)
                .style(egui_plot::LineStyle::dashed_loose())
                .width(3.);
            ui.line(line);
            let points = egui_plot::Points::new(target)
                .name(NAME)
                .color(Color32::BLUE)
                .filled(false)
                .radius(5.);
            ui.points(points);
        };
        match &self.target {
            io::Curve::P(t) => draw_curve(t.clone()),
            io::Curve::M(t) => {
                let scale = bound.width().min(bound.height()) / 2.;
                let (curve, pose): (Vec<_>, Vec<_>) = t
                    .iter()
                    .map(|(p, v)| (p, std::array::from_fn(|i| p[i] + scale * v[i])))
                    .unzip();
                for (p, v) in zip(&curve, &pose) {
                    draw_curve(vec![*p, *v]);
                }
                draw_curve(curve);
                draw_curve(pose);
            }
            io::Curve::S(t) => draw_curve(t.iter().map(|&[x, y, _]| [x, y]).collect()),
        }
    }

    fn on_click_canvas(&mut self, ui: &mut egui_plot::PlotUi, lnk: &Linkages) {
        let p = ui.pointer_coordinate().unwrap();
        match &mut self.target {
            io::Curve::P(t) => t.push([p.x, p.y]),
            io::Curve::M(t) => t.push(([p.x, p.y], [0., 0.])),
            io::Curve::S(t) => {
                // FIXME: Try block
                if let Some(c) = (|| {
                    let [sx, sy, sz, r] = lnk.projs.current_sphere()?;
                    let dx = p.x - sx;
                    let dy = p.y - sy;
                    (dx.hypot(dy) <= r).then_some([p.x, p.y, r * r - dx * dx - dy * dy + sz])
                })() {
                    t.push(c);
                } else {
                    let p = egui_plot::Points::new([p.x, p.y])
                        .shape(egui_plot::MarkerShape::Cross)
                        .color(Color32::RED)
                        .radius(30.);
                    ui.points(p);
                }
            }
        }
    }

    // Cache the visualization of atlas
    fn atlas_vis_cache(&mut self) {
        fn pca<M, const N: usize, const D: usize>(
            atlas: &atlas::Atlas<M, N, D>,
            is_sphere: bool,
        ) -> Vec<AtlasVis> {
            use smartcore::decomposition::pca::PCA;
            let reduced = PCA::fit(atlas.data(), Default::default())
                .unwrap()
                .transform(atlas.data())
                .unwrap();
            zip(atlas.open_iter(), reduced.rows())
                .map(|(is_open, pt)| AtlasVis { pt: [pt[0], pt[1]], is_open, is_sphere })
                .collect()
        }

        self.atlas_vis
            .reserve(self.atlas.as_fb().len() + self.atlas.as_sfb().len());
        if !self.atlas.as_fb().is_empty() {
            self.atlas_vis.extend(pca(self.atlas.as_fb(), false));
        }
        if !self.atlas.as_sfb().is_empty() {
            self.atlas_vis.extend(pca(self.atlas.as_sfb(), true));
        }
    }

    fn atlas_vis(&mut self, ui: &mut Ui) {
        if !self.atlas_vis_open {
            return;
        }
        let mut f = |name, title, draw_sphere| {
            Window::new(title)
                .open(&mut self.atlas_vis_open)
                .show(ui.ctx(), |ui| {
                    static_plot(name).view_aspect(1.).show(ui, |ui| {
                        for &AtlasVis { pt, is_open, is_sphere } in &self.atlas_vis {
                            if is_sphere != draw_sphere {
                                continue;
                            }
                            let (name, color) = if is_open {
                                ("Open Curve", Color32::RED)
                            } else {
                                ("Closed Curve", Color32::BLUE)
                            };
                            ui.points(egui_plot::Points::new(pt).color(color).name(name));
                        }
                    });
                });
        };
        f("atlas_vis_planar", "☁ Planar Data Visualization", false);
        f(
            "atlas_vis_spherical",
            "☁ Spherical Data Visualization",
            true,
        );
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
        let task = TaskInProg::new(task);
        self.task_queue.push(task.clone());
        let alg = self.alg.clone();
        let cfg = self.cfg.clone();
        let target = match self.target.clone() {
            io::Curve::P(t) => Target::fb(t.into(), None, Some(self.atlas.as_fb())),
            io::Curve::M(t) => Target::mfb(t.into(), None),
            io::Curve::S(t) => Target::sfb(t.into(), None, Some(self.atlas.as_sfb())),
        };
        let queue = lnk.projs.queue();
        let stop = {
            let pg = task.pg.clone();
            let finish = 1f32.to_bits();
            move || pg.load(SeqCst) == finish
        };
        let total_gen = cfg.gen;
        let t0 = Instant::now();
        let s = syn_cmd::Solver::new(alg, target, cfg, stop, move |best_f, gen| {
            pg_set(&task.pg, gen as f32 / total_gen as f32);
            let mut task = task.task.lock().unwrap();
            task.conv.push(best_f);
            task.time = t0.elapsed();
        });
        let f = move || queue.push(None, s.solve());
        #[cfg(not(target_arch = "wasm32"))]
        mh::rayon::spawn(f);
        #[cfg(target_arch = "wasm32")]
        f(); // Block
    }
}

fn pg_get(pg: &AtomicU32) -> f32 {
    f32::from_bits(pg.load(Relaxed))
}

fn pg_set(pg: &AtomicU32, v: f32) {
    pg.store(v.to_bits(), Relaxed);
}
