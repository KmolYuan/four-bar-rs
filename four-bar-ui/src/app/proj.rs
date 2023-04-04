use super::{io, link::Cfg, widgets::*};
use eframe::egui::*;
use four_bar::{csv::dump_csv, FourBar, SFourBar};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    f64::consts::TAU,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

mod undo;

const JOINT_COLOR: Color32 = Color32::from_rgb(93, 69, 56);
const LINK_COLOR: Color32 = Color32::from_rgb(165, 151, 132);

macro_rules! hotkey {
    ($ui:ident, $($mod:ident)|+, $key:ident) => {
        $ui.ctx().input_mut(|s| s.consume_key($(Modifiers::$mod)|+, Key::$key))
    };
}

fn pre_open(file: impl AsRef<Path>) -> Option<FourBar> {
    if cfg!(target_arch = "wasm32") {
        None
    } else {
        std::fs::read_to_string(file)
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
    }
}

fn filename(path: &Path) -> Cow<str> {
    path.file_name().unwrap().to_string_lossy()
}

fn draw_link(ui: &mut plot::PlotUi, points: &[[f64; 2]], is_main: bool) {
    let width = if is_main { 3. } else { 1. };
    if points.len() == 2 {
        let line = plot::Line::new(points.to_vec())
            .width(width)
            .color(LINK_COLOR);
        ui.line(line);
    } else {
        let polygon = plot::Polygon::new(points.to_vec())
            .width(width)
            .fill_alpha(if is_main { 0.8 } else { 0.2 })
            .color(LINK_COLOR);
        ui.polygon(polygon);
    }
}

fn plot_values(ui: &mut plot::PlotUi, values: &[(f64, [f64; 3])], sym: &str, use_rad: bool) {
    let mut v2 = Vec::with_capacity(values.len());
    let mut v3 = Vec::with_capacity(values.len());
    let mut v4 = Vec::with_capacity(values.len());
    for &(x, [y2, y3, y4]) in values {
        let [x, y2, y3, y4] = if use_rad {
            [x, y2, y3, y4]
        } else {
            [x, y2, y3, y4].map(f64::to_degrees)
        };
        v2.push([x, y2]);
        v3.push([x, y3]);
        v4.push([x, y4]);
    }
    ui.line(plot::Line::new(v2).name(format!("{sym}2")));
    ui.line(plot::Line::new(v3).name(format!("{sym}3")));
    ui.line(plot::Line::new(v4).name(format!("{sym}4")));
}

fn angle_bound_btns(ui: &mut Ui, theta2: &mut f64, start: f64, end: f64) -> Response {
    ui.group(|ui| {
        fn copy_btn(ui: &mut Ui, start: f64, end: f64, suffix: &str) {
            ui.horizontal(|ui| {
                let s_str = format!("{start:.04}");
                if ui.selectable_label(false, &s_str).clicked() {
                    ui.output_mut(|s| s.copied_text = s_str);
                }
                let e_str = format!("{end:.04}");
                if ui.selectable_label(false, &e_str).clicked() {
                    ui.output_mut(|s| s.copied_text = e_str);
                }
                ui.label(suffix);
            });
        }
        ui.label("Click to copy angle bounds:");
        copy_btn(ui, start, end, "rad");
        copy_btn(ui, start.to_degrees(), end.to_degrees(), "deg");
        ui.horizontal(|ui| {
            let mut res1 = ui.button("➡ To Start");
            if res1.clicked() {
                res1.mark_changed();
                *theta2 = start;
            }
            let mut res2 = ui.button("➡ To End");
            if res2.clicked() {
                res2.mark_changed();
                *theta2 = end;
            }
            res1 | res2
        })
        .inner
    })
    .inner
}

// TODO: Support spherical four-bar
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
enum Fb {
    FourBar(FourBar),
    SFourBar(SFourBar),
}

impl Default for Fb {
    fn default() -> Self {
        Self::FourBar(FourBar::example())
    }
}

#[derive(Default, Deserialize, Serialize, PartialEq, Eq, Copy, Clone)]
pub(crate) enum Pivot {
    Driver,
    Follower,
    #[default]
    Coupler,
}

impl Pivot {
    const fn name(&self) -> &str {
        match self {
            Pivot::Driver => "Driver",
            Pivot::Follower => "Follower",
            Pivot::Coupler => "Coupler",
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Default)]
#[serde(default)]
struct Angles {
    theta2: f64,
    omega2: f64,
    alpha2: f64,
}

#[derive(Default)]
struct Cache {
    changed: bool,
    defect: bool,
    has_closed_curve: bool,
    joints: [[f64; 2]; 5],
    curves: Vec<[[f64; 2]; 3]>,
    dynamics: Vec<(f64, [[f64; 3]; 3])>,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct ProjInner {
    path: Option<PathBuf>,
    fb: FourBar,
    angles: Angles,
    hide: bool,
    angle_open: bool,
    use_rad: bool,
    #[serde(skip)]
    cache: Cache,
    #[serde(skip)]
    undo: undo::Undo<undo::FourBarDelta>,
}

impl Default for ProjInner {
    fn default() -> Self {
        Self {
            path: Default::default(),
            fb: FourBar::example(),
            angles: Default::default(),
            hide: false,
            angle_open: false,
            use_rad: false,
            cache: Cache { changed: true, ..Cache::default() },
            undo: Default::default(),
        }
    }
}

impl ProjInner {
    fn show(&mut self, ui: &mut Ui, pivot: &mut Pivot, cfg: &Cfg) {
        path_label(ui, "🖹", self.path.as_ref(), "Untitled");
        ui.label("Linkage type:");
        ui.label(self.fb.ty().name());
        if self.cache.defect {
            ui.label(RichText::new("This linkage has defect!").color(Color32::RED));
        }
        if self.cache.has_closed_curve {
            ui.label(RichText::new("This linkage has a closed curve.").color(Color32::GREEN));
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.hide, "Hide 👁");
            let enabled = self.undo.able_undo();
            if ui
                .add_enabled(enabled, Button::new("⮪ Undo"))
                .on_hover_text("Ctrl+Z")
                .clicked()
                || hotkey!(ui, CTRL, Z)
            {
                self.undo.undo(&mut self.fb);
                self.cache.changed = true;
            }
            let enabled = self.undo.able_redo();
            if ui
                .add_enabled(enabled, Button::new("⮫ Redo"))
                .on_hover_text("Ctrl+Shift+Z | Ctrl+Y")
                .clicked()
                || hotkey!(ui, CTRL, Y)
                || hotkey!(ui, CTRL | SHIFT, Z)
            {
                self.undo.redo(&mut self.fb);
                self.cache.changed = true;
            }
            if small_btn(ui, "🗑", "Clear undo") {
                self.undo.clear();
            }
        });
        ui.add_enabled_ui(!self.hide, |ui| self.ui(ui, pivot, cfg));
        Window::new("⚽ Dynamics")
            .open(&mut self.angle_open)
            .vscroll(true)
            .show(ui.ctx(), |ui| {
                let res = angle(ui, "Omega: ", &mut self.angles.omega2, "/s")
                    | angle(ui, "Alpha: ", &mut self.angles.alpha2, "/s²");
                self.cache.changed |= res.changed();
                ui.checkbox(&mut self.use_rad, "Plot radians");
                for (i, (id, symbol, title)) in [
                    ("plot_theta", "theta", "Angle"),
                    ("plot_omega", "omega", "Angular Velocity"),
                    ("plot_alpha", "alpha", "Angular Acceleration"),
                ]
                .into_iter()
                .enumerate()
                {
                    ui.heading(title);
                    let values = self
                        .cache
                        .dynamics
                        .iter()
                        .map(|(x, t)| (*x, t[i]))
                        .collect::<Vec<_>>();
                    plot::Plot::new(id)
                        .legend(Default::default())
                        .height(200.)
                        .show(ui, |ui| plot_values(ui, &values, symbol, self.use_rad));
                }
            });
        self.undo.fetch(&self.fb);
    }

    fn ui(&mut self, ui: &mut Ui, pivot: &mut Pivot, cfg: &Cfg) {
        fn get_curve(pivot: Pivot, fb: &FourBar, n: usize) -> Vec<[f64; 2]> {
            let curve = fb.curves(n).into_iter();
            match pivot {
                Pivot::Driver => curve.map(|[c, _, _]| c).collect(),
                Pivot::Follower => curve.map(|[_, c, _]| c).collect(),
                Pivot::Coupler => curve.map(|[_, _, c]| c).collect(),
            }
        }
        ui.heading("Curve");
        ui.horizontal(|ui| {
            ComboBox::from_label("")
                .selected_text(pivot.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(pivot, Pivot::Coupler, Pivot::Coupler.name());
                    ui.selectable_value(pivot, Pivot::Driver, Pivot::Driver.name());
                    ui.selectable_value(pivot, Pivot::Follower, Pivot::Follower.name());
                });
            if small_btn(ui, "💾", "Save") {
                io::save_csv_ask(&get_curve(*pivot, &self.fb, cfg.res));
            }
            if small_btn(ui, "🗐", "Copy") {
                let t = dump_csv(get_curve(*pivot, &self.fb, cfg.res)).unwrap();
                ui.output_mut(|s| s.copied_text = t);
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("Offset");
            if ui
                .add_enabled(!self.fb.is_aligned(), Button::new("Reset"))
                .on_hover_text("Reset the translation and rotation offset")
                .clicked()
            {
                self.fb.align();
                self.cache.changed = true;
            }
            if ui
                .button("Normalize")
                .on_hover_text("Remove offset, then scale by the driver link")
                .clicked()
            {
                self.fb.normalize();
                self.cache.changed = true;
            }
        });
        let mut res = unit(ui, "X Offset: ", self.fb.p0x_mut(), cfg.int)
            | unit(ui, "Y Offset: ", self.fb.p0y_mut(), cfg.int)
            | angle(ui, "Rotation: ", self.fb.a_mut(), "");
        ui.separator();
        ui.heading("Parameters");
        res |= link(ui, "Ground: ", self.fb.l0_mut(), cfg.int)
            | link(ui, "Driver: ", self.fb.l1_mut(), cfg.int)
            | link(ui, "Coupler: ", self.fb.l2_mut(), cfg.int)
            | link(ui, "Follower: ", self.fb.l3_mut(), cfg.int)
            | link(ui, "Extended: ", self.fb.l4_mut(), cfg.int)
            | angle(ui, "Angle: ", self.fb.g_mut(), "")
            | ui.checkbox(self.fb.inv_mut(), "Invert follower and coupler");
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("Angle");
            if ui
                .add_enabled(self.angles != Default::default(), Button::new("Reset"))
                .clicked()
            {
                self.angles = Default::default();
                self.cache.changed = true;
            }
        });
        if let Some([start, end]) = self.fb.angle_bound() {
            res |= angle_bound_btns(ui, &mut self.angles.theta2, start, end);
        }
        ui.horizontal(|ui| {
            res |= ui
                .group(|ui| angle(ui, "Theta: ", &mut self.angles.theta2, ""))
                .inner;
            self.cache.changed |= res.changed();
            toggle_btn(ui, &mut self.angle_open, "⚽ Dynamics");
        });
        ui.separator();
        ui.heading("Figure");
        ui.label("Plot linkage and its coupler curve.");
        if ui.button("💾 Save Linkage").clicked() {
            let curve = get_curve(Pivot::Coupler, &self.fb, cfg.res);
            let opt = four_bar::plot2d::Opt::from(&self.fb)
                .angle(self.angles.theta2)
                .inner(cfg.plot.clone());
            io::save_curve_ask([("Coupler Curve", curve.as_slice())], opt, "fig.svg");
        }
        self.cache(cfg.res);
    }

    pub(crate) fn cache(&mut self, res: usize) {
        if self.cache.changed {
            self.cache_inner(res);
        }
    }

    fn cache_inner(&mut self, res: usize) {
        // Recalculation
        self.cache.changed = false;
        self.cache.joints = self.fb.pos(self.angles.theta2);
        self.cache.defect = self.fb.has_defect();
        self.cache.has_closed_curve = self.fb.has_closed_curve();
        self.cache.curves = self.fb.curves(res);
        let step = TAU / res as f64;
        self.cache.dynamics = self
            .cache
            .curves
            .iter()
            .enumerate()
            .map(|(i, [[x2, y2], [x3, y3], _])| {
                let [x0, y0] = self.cache.joints[0];
                let [x1, y1] = self.cache.joints[1];
                let theta2 = (y2 - y0).atan2(x2 - x0);
                let theta3 = (y3 - y2).atan2(x3 - x2);
                let theta4 = (y3 - y1).atan2(x3 - x1);
                let theta = [theta2, theta3, theta4];
                let omega2 = self.angles.omega2;
                let omega3 = -omega2 * self.fb.l1() * (theta4 - theta2).sin()
                    / (self.fb.l2() * (theta4 - theta3).sin() + f64::EPSILON);
                let omega4 = omega2 * self.fb.l1() * (theta3 - theta2).sin()
                    / (self.fb.l3() * (theta3 - theta4).sin() + f64::EPSILON);
                let omega = [omega2, omega3, omega4];
                let alpha2 = self.angles.alpha2;
                let alpha3 = (-self.fb.l1() * alpha2 * (theta4 - theta2).sin()
                    + self.fb.l1() * omega2 * omega2 * (theta4 - theta2).cos()
                    + self.fb.l2() * omega3 * omega3 * (theta4 - theta3).cos()
                    - self.fb.l3() * omega4 * omega4)
                    / self.fb.l2()
                    * (theta4 - theta3).sin();
                let alpha4 = (self.fb.l1() * alpha2 * (theta3 - theta2).sin()
                    - self.fb.l1() * omega2 * omega2 * (theta3 - theta2).cos()
                    + self.fb.l3() * omega4 * omega4 * (theta3 - theta4).cos()
                    - self.fb.l2() * omega3 * omega3)
                    / self.fb.l3()
                    * (theta3 - theta4).sin();
                let alpha = [alpha2, alpha3, alpha4];
                (i as f64 * step, [theta, omega, alpha])
            })
            .collect();
    }

    fn plot(&self, ui: &mut plot::PlotUi, ind: usize, id: usize) {
        if self.hide {
            return;
        }
        let is_main = ind == id;
        draw_link(ui, &[self.cache.joints[0], self.cache.joints[2]], is_main);
        draw_link(ui, &[self.cache.joints[1], self.cache.joints[3]], is_main);
        draw_link(ui, &self.cache.joints[2..], is_main);
        let float_j = plot::Points::new(self.cache.joints[2..].to_vec())
            .radius(5.)
            .color(JOINT_COLOR);
        let fixed_j = plot::Points::new(self.cache.joints[..2].to_vec())
            .radius(10.)
            .shape(plot::MarkerShape::Up)
            .color(JOINT_COLOR);
        ui.points(float_j);
        ui.points(fixed_j);
        for (i, name) in ["Driver joint", "Follower joint", "Coupler joint"]
            .into_iter()
            .enumerate()
        {
            let iter = self.cache.curves.iter().map(|c| c[i]).collect::<Vec<_>>();
            ui.line(plot::Line::new(iter).name(name).width(3.));
        }
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub(crate) struct Project(Arc<RwLock<ProjInner>>);

impl Project {
    fn new(path: Option<PathBuf>, fb: FourBar) -> Self {
        let inner = ProjInner { path, fb, ..Default::default() };
        Self(Arc::new(RwLock::new(inner)))
    }

    fn set_path(&self, path: PathBuf) {
        self.0.write().unwrap().path = Some(path);
    }

    pub(crate) fn path(&self) -> Option<PathBuf> {
        self.0.read().unwrap().path.clone()
    }

    pub(crate) fn pre_open(&self) {
        let mut proj = self.0.write().unwrap();
        if let Some(path) = &proj.path {
            if let Some(fb) = pre_open(path) {
                proj.fb = fb;
            } else {
                proj.path = None;
            }
        }
    }

    pub(crate) fn name(&self) -> String {
        if let Some(path) = &self.0.read().unwrap().path {
            let name = filename(path);
            if name.ends_with(".ron") {
                name.to_string()
            } else {
                name.to_string() + ".ron"
            }
        } else {
            "untitled.ron".to_string()
        }
    }

    fn save(&self) {
        let proj = self.0.read().unwrap();
        if let Some(path) = &proj.path {
            io::save_ron(&proj.fb, path);
        } else {
            drop(proj);
            self.save_as();
        }
    }

    fn save_as(&self) {
        let name = self.name();
        let (fb, _) = self.four_bar_state();
        let proj_cloned = self.clone();
        io::save_ron_ask(&fb, &name, move |path| proj_cloned.set_path(path));
    }

    fn show(&self, ui: &mut Ui, pivot: &mut Pivot, cfg: &Cfg) {
        self.0.write().unwrap().show(ui, pivot, cfg);
    }

    fn plot(&self, ui: &mut plot::PlotUi, i: usize, id: usize) {
        self.0.read().unwrap().plot(ui, i, id);
    }

    fn four_bar_state(&self) -> (FourBar, f64) {
        let proj = self.0.read().unwrap();
        (proj.fb.clone(), proj.angles.theta2)
    }

    pub(crate) fn clone_curve(&self) -> Vec<[f64; 2]> {
        self.0
            .read()
            .unwrap()
            .cache
            .curves
            .iter()
            .map(|[_, _, c]| *c)
            .collect()
    }

    pub(crate) fn cache(&self, res: usize) {
        self.0.write().unwrap().cache(res);
    }

    fn request_cache(&self) {
        self.0.write().unwrap().cache.changed = true;
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub(crate) struct Queue(Arc<RwLock<Vec<Project>>>);

impl Queue {
    pub(crate) fn push(&self, path: Option<PathBuf>, fb: FourBar) {
        self.0.write().unwrap().push(Project::new(path, fb));
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Projects {
    list: Vec<Project>,
    queue: Queue,
    pivot: Pivot,
    curr: usize,
}

impl Projects {
    pub(crate) fn push(&mut self, path: Option<PathBuf>, fb: FourBar) {
        // Prevent opening duplicate project
        if match &path {
            None => true,
            Some(path) => !self
                .list
                .iter()
                .any(|p| matches!(p.path(), Some(path_old) if &path_old == path)),
        } {
            self.queue.push(path, fb);
        }
    }

    pub(crate) fn push_example(&self) {
        self.queue.0.write().unwrap().push(Project::default());
    }

    pub(crate) fn pre_open(&mut self, file: PathBuf) {
        if let Some(fb) = pre_open(&file) {
            self.push(Some(file), fb);
        }
    }

    pub(crate) fn queue(&self) -> Queue {
        self.queue.clone()
    }

    pub(crate) fn poll(&mut self, ctx: &Context, n: usize) {
        #[cfg(not(target_arch = "wasm32"))]
        ctx.input(|s| {
            for file in s.raw.dropped_files.iter() {
                if let Some(path) = &file.path {
                    self.pre_open(path.clone());
                }
            }
        });
        let len = self.queue.0.read().unwrap().len();
        if len > 0 {
            self.list.reserve(len);
            while let Some(proj) = self.queue.0.write().unwrap().pop() {
                proj.cache(n);
                self.list.push(proj);
            }
            self.curr = self.len() - 1;
            ctx.request_repaint();
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, cfg: &Cfg) {
        ui.horizontal(|ui| {
            if ui.button("🖴 Load").clicked() || hotkey!(ui, CTRL, O) {
                let q = self.queue();
                io::open_ron(move |path, fb| q.push(Some(path), fb));
            }
            if ui.button("🗋 New").clicked() {
                self.push_example();
            }
        });
        if self.select(ui, true) {
            self.list[self.curr].show(ui, &mut self.pivot, cfg);
        } else {
            ui.heading("No project here!");
            ui.label("Please open or create a project.");
        }
    }

    pub(crate) fn select(&mut self, ui: &mut Ui, show_btn: bool) -> bool {
        if self.is_empty() {
            return false;
        }
        ui.horizontal(|ui| {
            ComboBox::from_label("")
                .show_index(ui, &mut self.curr, self.list.len(), |i| self.list[i].name());
            if !show_btn {
                return;
            }
            if small_btn(ui, "💾", "Save (Ctrl+S)") || hotkey!(ui, CTRL, S) {
                self.list[self.curr].save();
            }
            if small_btn(ui, "💾 Save As", "Ctrl+Shift+S") || hotkey!(ui, CTRL | SHIFT, S) {
                self.list[self.curr].save_as();
            }
            if small_btn(ui, "✖", "Close (Ctrl+W)") || hotkey!(ui, CTRL, W) {
                self.list.remove(self.curr);
                if self.curr > 0 {
                    self.curr -= 1;
                }
            }
        });
        !self.is_empty()
    }

    pub(crate) fn four_bar_state(&self) -> (FourBar, f64) {
        self.list[self.curr].four_bar_state()
    }

    pub(crate) fn current_curve(&self) -> Vec<[f64; 2]> {
        self.list[self.curr].clone_curve()
    }

    pub(crate) fn request_cache(&self) {
        self.list[self.curr].request_cache();
    }

    pub(crate) fn plot(&self, ui: &mut plot::PlotUi) {
        for (i, proj) in self.list.iter().enumerate() {
            proj.plot(ui, i, self.curr);
        }
    }
}

impl Deref for Projects {
    type Target = Vec<Project>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for Projects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}
