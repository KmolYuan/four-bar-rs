use super::{io, link::Cfg, widgets::*};
use eframe::egui::*;
use four_bar::{csv::dump_csv, CurveGen as _, *};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

mod undo;

const JOINT_COLOR: Color32 = Color32::from_rgb(93, 69, 56);
const LINK_COLOR: Color32 = Color32::from_rgb(165, 151, 132);

macro_rules! hotkey {
    ($ui:ident, $mod1:ident + $key:ident) => {
        hotkey!(@$ui, Modifiers::$mod1, Key::$key)
    };

    ($ui:ident, $mod1:ident + $mod2:ident + $key:ident) => {
        hotkey!(@$ui, Modifiers::$mod1 | Modifiers::$mod2, Key::$key)
    };

    (@$ui:ident, $arg1:expr, $arg2:expr) => {
        $ui.ctx().input_mut(|s| s.consume_key($arg1, $arg2))
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

fn filename(path: &Path) -> std::borrow::Cow<str> {
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

struct Cache<D: efd::EfdDim> {
    changed: bool,
    angle_bound: AngleBound,
    joints: Option<[efd::Coord<D>; 5]>,
    curves: Vec<[efd::Coord<D>; 3]>,
}

impl<D: efd::EfdDim> Default for Cache<D> {
    fn default() -> Self {
        Self {
            changed: true,
            angle_bound: AngleBound::Invalid,
            joints: None,
            curves: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct ProjInner {
    path: Option<PathBuf>,
    fb: FourBar,
    angle: f64,
    hide: bool,
    use_rad: bool,
    #[serde(skip)]
    cache: Cache<efd::D2>,
    #[serde(skip)]
    undo: undo::Undo<undo::FbDelta>,
}

impl Default for ProjInner {
    fn default() -> Self {
        Self {
            path: Default::default(),
            fb: FourBar::example(),
            angle: 0.,
            hide: false,
            use_rad: false,
            cache: Default::default(),
            undo: Default::default(),
        }
    }
}

impl ProjInner {
    fn show(&mut self, ui: &mut Ui, pivot: &mut Pivot, cfg: &Cfg) {
        path_label(ui, "🖹", self.path.as_ref(), "Untitled");
        ui.label("Linkage type:");
        ui.label(self.fb.ty().name());
        match self.cache.angle_bound {
            AngleBound::Closed => ui.label("This linkage has a closed curve."),
            AngleBound::Open(_, _) => ui.label("This linkage has an open curve."),
            AngleBound::Invalid => ui.label("This linkage is invalid."),
        };
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.hide, "Hide 👁");
            let enabled = self.undo.able_undo();
            if ui
                .add_enabled(enabled, Button::new("⮪ Undo"))
                .on_hover_text("Ctrl+Z")
                .clicked()
                || hotkey!(ui, CTRL + Z)
            {
                self.undo.undo(&mut self.fb);
                self.cache.changed = true;
            }
            let enabled = self.undo.able_redo();
            if ui
                .add_enabled(enabled, Button::new("⮫ Redo"))
                .on_hover_text("Ctrl+Shift+Z | Ctrl+Y")
                .clicked()
                || hotkey!(ui, CTRL + Y)
                || hotkey!(ui, CTRL + SHIFT + Z)
            {
                self.undo.redo(&mut self.fb);
                self.cache.changed = true;
            }
            if small_btn(ui, "🗑", "Clear undo") {
                self.undo.clear();
            }
        });
        ui.add_enabled_ui(!self.hide, |ui| self.ui(ui, pivot, cfg));
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
                .add_enabled(self.fb.buf[..3] != [0.; 3], Button::new("Reset"))
                .on_hover_text("Reset the translation and rotation offset")
                .clicked()
            {
                self.fb.buf[..3].iter_mut().for_each(|x| *x = 0.);
                self.cache.changed = true;
            }
            if ui
                .button("Normalize")
                .on_hover_text("Remove offset, then scale by the driver link")
                .clicked()
            {
                self.fb = self.fb.normalize::<_, NormFourBar>().denormalize();
                self.cache.changed = true;
            }
        });
        let mut res = unit(ui, "X Offset: ", self.fb.p0x_mut(), cfg.int)
            | unit(ui, "Y Offset: ", self.fb.p0y_mut(), cfg.int)
            | angle(ui, "Rotation: ", self.fb.a_mut(), "");
        ui.separator();
        ui.heading("Parameters");
        res |= nonzero_f(ui, "Ground: ", self.fb.l1_mut(), cfg.int)
            | nonzero_f(ui, "Driver: ", self.fb.l2_mut(), cfg.int)
            | nonzero_f(ui, "Coupler: ", self.fb.l3_mut(), cfg.int)
            | nonzero_f(ui, "Follower: ", self.fb.l4_mut(), cfg.int)
            | nonzero_f(ui, "Extended: ", self.fb.l5_mut(), cfg.int)
            | angle(ui, "Angle: ", self.fb.g_mut(), "")
            | ui.checkbox(self.fb.inv_mut(), "Invert follower and coupler");
        ui.separator();
        ui.heading("Angle");
        if let Some([start, end]) = self.cache.angle_bound.to_value() {
            res |= angle_bound_btns(ui, &mut self.angle, start, end);
        }
        ui.horizontal(|ui| {
            res |= ui
                .group(|ui| angle(ui, "Theta: ", &mut self.angle, ""))
                .inner;
            self.cache.changed |= res.changed();
        });
        self.cache(cfg.res);
    }

    pub(crate) fn cache(&mut self, res: usize) {
        if self.cache.changed {
            // Recalculation
            self.cache.changed = false;
            self.cache.joints = self.fb.pos(self.angle);
            self.cache.angle_bound = self.fb.angle_bound();
            self.cache.curves = self.fb.curves(res);
        }
    }

    fn plot(&self, ui: &mut plot::PlotUi, ind: usize, id: usize) {
        if self.hide {
            return;
        }
        let Some(joints) = self.cache.joints else { return };
        let is_main = ind == id;
        draw_link(ui, &[joints[0], joints[2]], is_main);
        draw_link(ui, &[joints[1], joints[3]], is_main);
        draw_link(ui, &joints[2..], is_main);
        let float_j = plot::Points::new(joints[2..].to_vec())
            .radius(5.)
            .color(JOINT_COLOR);
        let fixed_j = plot::Points::new(joints[..2].to_vec())
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
        self.0.write().unwrap().path.replace(path);
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
                proj.path.take();
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
        let fb = self.0.read().unwrap().fb.clone();
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
        (proj.fb.clone(), proj.angle)
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
            if ui.button("🖴 Load").clicked() || hotkey!(ui, CTRL + O) {
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
            if small_btn(ui, "💾", "Save (Ctrl+S)") || hotkey!(ui, CTRL + S) {
                self.list[self.curr].save();
            }
            if small_btn(ui, "💾 Save As", "Ctrl+Shift+S") || hotkey!(ui, CTRL + SHIFT + S) {
                self.list[self.curr].save_as();
            }
            if small_btn(ui, "✖", "Close (Ctrl+W)") || hotkey!(ui, CTRL + W) {
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

impl std::ops::Deref for Projects {
    type Target = Vec<Project>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl std::ops::DerefMut for Projects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}
