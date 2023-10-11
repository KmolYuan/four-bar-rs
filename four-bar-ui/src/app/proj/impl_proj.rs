use super::*;
use four_bar::{csv, efd, fb, NormFourBar, SNormFourBar};
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
    Fb(FbProj),
    SFb(SFbProj),
}

macro_rules! impl_method {
    ($(fn $method:ident($self:ident: $self_ty:ty $(, $v:ident: $ty:ty)*) $(-> $ret:ty)?;)+) => {$(
        pub(crate) fn $method($self: $self_ty $(, $v: $ty)*) $(-> $ret)? {
            match $self {
                Self::Fb(fb) => fb.$method($($v),*),
                Self::SFb(fb) => fb.$method($($v),*),
            }
        }
    )+};
}

impl Project {
    pub(crate) fn new(path: Option<PathBuf>, fb: io::Fb) -> Self {
        match fb {
            io::Fb::Fb(fb) => Self::Fb(FbProj::new(path, fb)),
            io::Fb::SFb(fb) => Self::SFb(SFbProj::new(path, fb)),
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
            Self::Fb(proj) => (proj.angle, io::Fb::Fb(proj.fb.clone())),
            Self::SFb(proj) => (proj.angle, io::Fb::SFb(proj.fb.clone())),
        }
    }

    pub(crate) fn curve(&self) -> io::Curve {
        match self {
            Self::Fb(proj) => io::Curve::P(proj.cache.curves.iter().map(|[.., c]| *c).collect()),
            Self::SFb(proj) => io::Curve::S(proj.cache.curves.iter().map(|[.., c]| *c).collect()),
        }
    }

    pub(crate) fn get_sphere(&self) -> Option<[f64; 4]> {
        match self {
            Self::SFb(proj) => Some(proj.fb.scr()),
            _ => None,
        }
    }

    pub(crate) fn proj_name(&self) -> String {
        let (prefix, mut name) = match self {
            Self::Fb(proj) => ("[P] ", proj.name()),
            Self::SFb(proj) => ("[S] ", proj.name()),
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

type FbProj = ProjInner<efd::D2, NormFourBar>;
type SFbProj = ProjInner<efd::D3, SNormFourBar>;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct ProjInner<D, M>
where
    D: efd::EfdDim,
    M: fb::Normalized<D>,
    M::De: fb::CurveGen<D> + undo::IntoDelta,
{
    path: Option<PathBuf>,
    fb: M::De,
    angle: f64,
    curve_range: Option<[f64; 2]>,
    curve_res: usize,
    hide: bool,
    #[serde(skip)]
    unsaved: bool,
    #[serde(skip)]
    cache: Cache<D>,
    #[serde(skip)]
    undo: undo::Undo<<M::De as undo::IntoDelta>::Delta>,
}

impl<D, M> Default for ProjInner<D, M>
where
    D: efd::EfdDim,
    M: fb::Normalized<D>,
    M::De: fb::CurveGen<D> + undo::IntoDelta + Default,
{
    fn default() -> Self {
        Self {
            path: Default::default(),
            fb: Default::default(),
            angle: 0.,
            curve_range: None,
            curve_res: 40,
            hide: false,
            unsaved: false,
            cache: Default::default(),
            undo: Default::default(),
        }
    }
}

struct Cache<D: efd::EfdDim> {
    changed: bool,
    angle_bound: fb::AngleBound,
    joints: Option<[efd::Coord<D>; 5]>,
    curves: Vec<[efd::Coord<D>; 3]>,
}

impl<D: efd::EfdDim> Default for Cache<D> {
    fn default() -> Self {
        Self {
            changed: true,
            angle_bound: fb::AngleBound::Invalid,
            joints: None,
            curves: Vec::new(),
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

impl<D, M> ProjInner<D, M>
where
    D: efd::EfdDim,
    M: fb::Normalized<D>,
    M::De: fb::CurveGen<D>
        + fb::PlanarLoop
        + undo::IntoDelta
        + ui::ProjUi
        + ui::ProjPlot<D>
        + PartialEq
        + Default
        + Serialize
        + serde::de::DeserializeOwned,
    efd::Coord<D>: Serialize,
{
    fn new(path: Option<PathBuf>, fb: M::De) -> Self {
        Self { path, fb, ..Self::default() }
    }

    fn show(&mut self, ui: &mut Ui, pivot: &mut Pivot, cfg: &Cfg) {
        use four_bar::fb::PlanarLoop as _;
        ui.horizontal(|ui| {
            if small_btn(ui, "🔗", "Share with Link") {
                let mut url = b"https://kmolyuan.github.io/four-bar-rs/?code=".to_vec();
                self.fb
                    .serialize(&mut ron::Serializer::new(&mut url, None).unwrap())
                    .unwrap();
                let url = String::from_utf8_lossy(&url);
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
        match self.cache.angle_bound {
            fb::AngleBound::Closed => ui.label("This linkage has a closed curve."),
            fb::AngleBound::Open(_, _) => ui.label("This linkage has an open curve."),
            fb::AngleBound::Invalid => ui.label("This linkage is invalid."),
        };
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
        use four_bar::fb::CurveGen as _;
        ui.heading("Curve");
        check_on(ui, "In range", &mut self.curve_range, |ui, [start, end]| {
            angle(ui, "start: ", start, "") | angle(ui, "end: ", end, "")
        });
        nonzero_i(ui, "Resolution: ", &mut self.curve_res, 1);
        ui.horizontal(|ui| {
            const OPTS: [Pivot; 3] = [Pivot::Coupler, Pivot::Driver, Pivot::Follower];
            combo_enum(ui, "pivot", pivot, OPTS, |e| e.name());
            let get_curve = |pivot, fb: &M::De| -> Vec<_> {
                let curve = if let Some([start, end]) = self.curve_range {
                    fb.curves_in(start, end, self.curve_res).into_iter()
                } else {
                    fb.curves(self.curve_res).into_iter()
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
            }
        });
        let mut res = ui::ProjUi::proj_ui(&mut self.fb, ui, cfg);
        self.unsaved |= res.changed();
        ui.separator();
        ui.heading("Angle");
        if let Some([start, end]) = self.cache.angle_bound.to_value() {
            res |= angle_bound_ui(ui, &mut self.angle, start, end);
        }
        res |= angle(ui, "Theta: ", &mut self.angle, "");
        self.cache.changed |= res.changed();
        self.cache(cfg.res);
    }

    fn cache(&mut self, res: usize) {
        use four_bar::fb::{CurveGen as _, PlanarLoop as _};
        if self.cache.changed {
            // Recalculation
            self.cache.changed = false;
            self.cache.joints = self.fb.pos(self.angle);
            self.cache.angle_bound = self.fb.angle_bound();
            self.cache.curves = self.fb.curves(res);
        }
    }

    fn plot(&self, ui: &mut egui_plot::PlotUi, ind: usize, id: usize) {
        use ui::ProjPlot as _;
        if !self.hide {
            let joints = self.cache.joints.as_ref();
            self.fb.proj_plot(ui, joints, &self.cache.curves, ind == id);
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
        if cfg!(target_arch = "wasm32") {
            return;
        }
        // FIXME: Try block, ignore errors
        (|| {
            let r = std::fs::File::open(self.path.as_ref()?).ok()?;
            if self.fb != ron::de::from_reader(r).ok()? {
                self.unsaved = true;
            }
            Some(())
        })();
    }

    fn set_path(&mut self, path: PathBuf) {
        self.path.replace(path);
    }

    fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn is_unsaved(&self) -> bool {
        self.unsaved
    }

    fn mark_saved(&mut self) {
        self.unsaved = false;
    }
}
