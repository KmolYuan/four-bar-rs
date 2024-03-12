use super::*;
use four_bar::*;
use std::path::Path;

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
pub(crate) use hotkey;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum Project {
    P(FbProj),
    M(MFbProj),
    S(SFbProj),
}

macro_rules! impl_method {
    ($(fn $method:ident($self:ident: $self_ty:ty $(, $v:ident: $ty:ty)*) $(-> $ret:ty)?;)+) => {$(
        pub(crate) fn $method($self: $self_ty $(, $v: $ty)*) $(-> $ret)? {
            match $self {
                Self::P(fb) => fb.$method($($v),*),
                Self::M(fb) => fb.$method($($v),*),
                Self::S(fb) => fb.$method($($v),*),
            }
        }
    )+};
}

impl Project {
    pub(crate) fn new(path: Option<PathBuf>, fb: io::Fb) -> Self {
        match fb {
            io::Fb::P(fb) => Self::P(FbProj::new(path, fb)),
            io::Fb::M(fb) => Self::M(MFbProj::new(path, fb)),
            io::Fb::S(fb) => Self::S(SFbProj::new(path, fb)),
        }
    }

    pub(crate) fn pre_open(path: PathBuf) -> Option<Self> {
        if cfg!(target_arch = "wasm32") {
            return None;
        }
        let fb = ron::de::from_reader(std::fs::File::open(&path).ok()?).ok()?;
        Some(Self::new(Some(path), fb))
    }

    pub(crate) fn fb_state(&self) -> (f64, io::Fb) {
        match self {
            Self::P(proj) => (proj.angle, io::Fb::P(proj.fb.clone())),
            Self::M(proj) => (proj.angle, io::Fb::M(proj.fb.clone())),
            Self::S(proj) => (proj.angle, io::Fb::S(proj.fb.clone())),
        }
    }

    pub(crate) fn curve(&self) -> io::Curve {
        match self {
            Self::P(proj) => io::Curve::P(proj.curve()),
            Self::M(proj) => io::Curve::P(proj.curve()),
            Self::S(proj) => io::Curve::S(proj.curve()),
        }
    }

    pub(crate) fn get_sphere(&self) -> Option<[f64; 4]> {
        match self {
            Self::S(proj) => Some(proj.fb.scr()),
            _ => None,
        }
    }

    pub(crate) fn proj_name(&self) -> String {
        let (prefix, mut name) = match self {
            Self::P(proj) => ("[P] ", proj.name()),
            Self::M(proj) => ("[M] ", proj.name()),
            Self::S(proj) => ("[S] ", proj.name()),
        };
        name.insert_str(0, prefix);
        name
    }

    impl_method! {
        fn show(self: &mut Self, ui: &mut Ui, pivot: &mut Pivot, cfg: &Cfg);
        fn plot(self: &Self, ui: &mut egui_plot::PlotUi, ind: usize, id: usize);
        fn cache(self: &mut Self, res: usize);
        fn request_cache(self: &mut Self);
        fn name(self: &Self) -> String;
        fn preload(self: &mut Self);
        fn set_path(self: &mut Self, path: PathBuf);
        fn path(self: &Self) -> Option<&Path>;
        fn is_unsaved(self: &Self) -> bool;
        fn mark_saved(self: &mut Self);
    }
}

type FbProj = ProjInner<NormFourBar, 2>;
type MFbProj = ProjInner<MNormFourBar, 2>;
type SFbProj = ProjInner<SNormFourBar, 3>;

#[derive(Deserialize, Serialize)]
pub(crate) struct ProjInner<M, const D: usize>
where
    M: mech::Normalized<D>,
    M::De: mech::CurveGen<D> + undo::IntoDelta,
    efd::U<D>: efd::EfdDim<D>,
{
    path: Option<PathBuf>,
    fb: M::De,
    angle: f64,
    c_range: Option<[f64; 2]>,
    res: usize,
    hide: bool,
    #[serde(skip)]
    unsaved: bool,
    #[serde(skip)]
    cache: Cache<D>,
    #[serde(skip)]
    undo: undo::Undo<<M::De as undo::IntoDelta>::Delta>,
}

impl<M, const D: usize> Default for ProjInner<M, D>
where
    M: mech::Normalized<D>,
    M::De: mech::CurveGen<D> + undo::IntoDelta + Default,
    efd::U<D>: efd::EfdDim<D>,
{
    fn default() -> Self {
        Self {
            path: Default::default(),
            fb: Default::default(),
            angle: 0.,
            c_range: None,
            res: 40,
            hide: false,
            unsaved: false,
            cache: Default::default(),
            undo: Default::default(),
        }
    }
}

pub(crate) struct Cache<const D: usize> {
    changed: bool,
    angle_bound: mech::AngleBound,
    pub(crate) joints: Option<[[f64; D]; 5]>,
    pub(crate) curves: Vec<[[f64; D]; 3]>,
    pub(crate) stat_curves: Vec<Vec<[f64; D]>>,
}

impl<const D: usize> Default for Cache<D> {
    fn default() -> Self {
        Self {
            changed: true,
            angle_bound: mech::AngleBound::Invalid,
            joints: None,
            curves: Vec::new(),
            stat_curves: Vec::new(),
        }
    }
}

fn angle_bound_ui(ui: &mut Ui, theta2: &mut f64, start: f64, end: f64) -> Response {
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
    let res = ui.collapsing("Angle bound", |ui| {
        ui.label("Click to copy values:");
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
    });
    res.body_returned.unwrap_or(res.header_response)
}

impl<M, const D: usize> ProjInner<M, D>
where
    M: mech::Normalized<D>,
    M::De: mech::CurveGen<D>
        + mech::Statable
        + undo::IntoDelta
        + fb_ui::ProjUi
        + fb_ui::ProjPlot<D>
        + PartialEq
        + Default
        + Serialize
        + serde::de::DeserializeOwned,
    [f64; D]: Serialize,
    efd::U<D>: efd::EfdDim<D>,
{
    fn new(path: Option<PathBuf>, fb: M::De) -> Self {
        Self { path, fb, ..Self::default() }
    }

    fn show(&mut self, ui: &mut Ui, pivot: &mut Pivot, cfg: &Cfg) {
        use four_bar::mech::Statable as _;
        ui.horizontal(|ui| {
            if small_btn(ui, "🔗", "Share with Link") {
                let mut url = "https://kmolyuan.github.io/four-bar-rs/?code=".to_string();
                self.fb
                    .serialize(&mut ron::Serializer::new(&mut url, None).unwrap())
                    .unwrap();
                ui.ctx().open_url(OpenUrl::new_tab(url));
            }
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(path) = &self.path {
                use crate::io::Alert;
                if small_btn(ui, "🖴", "Reload from Disk") {
                    std::fs::File::open(path).alert_then("Failed to open file.", |r| {
                        ron::de::from_reader(r)
                            .alert_then("Failed to deserialize file.", |fb| self.fb = fb);
                    });
                }
            }
            path_label(ui, "🖹", self.path.as_ref(), "Unsaved");
        });
        ui.label("Linkage type:");
        ui.label(self.fb.ty().name());
        ui.label(self.cache.angle_bound.description());
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.hide, "Hide 👁");
            if ui
                .add_enabled(self.undo.able_undo(), Button::new("⮪ Undo"))
                .on_hover_text("Ctrl+Z")
                .clicked()
                || hotkey!(ui, CTRL + Z)
            {
                self.undo.undo(&mut self.fb);
                self.cache.changed = true;
            }
            if ui
                .add_enabled(self.undo.able_redo(), Button::new("⮫ Redo"))
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
        ui.heading("Curve");
        check_on(ui, "With range", &mut self.c_range, |ui, [start, end]| {
            ui.vertical(|ui| angle(ui, "start: ", start, "") | angle(ui, "end: ", end, ""))
                .inner
        });
        nonzero_i(ui, "Resolution: ", &mut self.res, 1);
        ui.horizontal(|ui| {
            const OPTS: [Pivot; 3] = [Pivot::Coupler, Pivot::Driver, Pivot::Follower];
            combo_enum(ui, "pivot", pivot, OPTS, |e| e.name());
            let get_curve = |pivot, fb: &M::De| -> Vec<_> {
                use four_bar::mech::CurveGen as _;
                let curve = if let Some([start, end]) = self.c_range {
                    fb.curves_in(start, end, self.res).into_iter()
                } else {
                    fb.curves(self.res).into_iter()
                };
                match pivot {
                    Pivot::Driver => curve.map(|[c, _, _]| c).collect(),
                    Pivot::Follower => curve.map(|[_, c, _]| c).collect(),
                    Pivot::Coupler => curve.map(|[_, _, c]| c).collect(),
                }
            };
            if small_btn(ui, "💾", "Save") {
                io::save_csv_ask(&get_curve(*pivot, &self.fb));
            }
            if small_btn(ui, "🗐", "Copy") {
                let t = csv::to_string(get_curve(*pivot, &self.fb)).unwrap();
                ui.output_mut(|s| s.copied_text = t);
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("Offset");
            if ui
                .button("Normalize")
                .on_hover_text("Remove offset, then scale by the driver link")
                .clicked()
            {
                M::normalize_inplace(&mut self.fb);
                self.cache.changed = true;
                self.unsaved = true;
            }
        });
        let mut res = fb_ui::ProjUi::proj_ui(&mut self.fb, ui, cfg);
        self.unsaved |= res.changed();
        ui.separator();
        ui.heading("Angle");
        if let Some([start, end]) = self.cache.angle_bound.to_value() {
            res |= angle_bound_ui(ui, &mut self.angle, start, end);
        }
        res |= angle(ui, "Theta: ", &mut self.angle, "");
        self.cache.changed |= res.changed();
        if self.cache.changed {
            self.cache(cfg.res);
        }
    }

    fn cache(&mut self, res: usize) {
        use four_bar::mech::{CurveGen as _, Statable as _};
        self.cache.changed = false;
        self.cache.joints = self.fb.pos(self.angle);
        self.cache.angle_bound = self.fb.angle_bound();
        self.cache.curves = self.fb.curves(res);
        self.cache.stat_curves = self
            .cache
            .angle_bound
            .get_states()
            .into_iter()
            .filter(|s| *s != self.fb.stat())
            .map(|s| self.fb.clone().with_stat(s).curve(res))
            .collect();
    }

    fn plot(&self, ui: &mut egui_plot::PlotUi, ind: usize, id: usize) {
        use fb_ui::ProjPlot as _;
        if !self.hide {
            self.fb.proj_plot(ui, &self.cache, ind == id);
        }
    }

    fn request_cache(&mut self) {
        self.cache.changed = true;
    }

    fn name(&self) -> String {
        if let Some(path) = &self.path {
            let name = path.file_name().unwrap().to_string_lossy();
            if name.ends_with(".ron") {
                name.to_string()
            } else {
                name.to_string() + ".ron"
            }
        } else {
            "untitled.ron".to_string()
        }
    }

    fn preload(&mut self) {
        // FIXME: Try block, ignore errors
        #[cfg(not(target_arch = "wasm32"))]
        (|| {
            let r = std::fs::File::open(self.path.as_ref()?).ok()?;
            if self.fb != ron::de::from_reader(r).ok()? {
                self.unsaved = true;
            }
            Some(())
        })();
    }

    fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn curve(&self) -> Vec<[f64; D]> {
        self.cache.curves.iter().map(|[.., c]| *c).collect()
    }

    fn is_unsaved(&self) -> bool {
        self.unsaved
    }

    fn mark_saved(&mut self) {
        self.unsaved = false;
    }
}
