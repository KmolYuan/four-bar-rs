use super::{io::Fb, *};
use crate::app::plotter::Curve;
use std::path::PathBuf;

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
pub(crate) enum ProjSwitch {
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

impl ProjSwitch {
    pub(crate) fn new_fb() -> Self {
        Self::Fb(FbProj::new(FourBar::example()))
    }

    pub(crate) fn new_sfb() -> Self {
        Self::SFb(SFbProj::new(SFourBar::example()))
    }

    pub(crate) fn new(path: Option<PathBuf>, fb: Fb) -> Self {
        match fb {
            Fb::Fb(fb) => Self::Fb(FbProj::new_with_path(path, fb)),
            Fb::SFb(fb) => Self::SFb(SFbProj::new_with_path(path, fb)),
        }
    }

    pub(crate) fn pre_open(path: impl AsRef<Path>) -> Option<Self> {
        if cfg!(target_arch = "wasm32") {
            return None;
        }
        match ron::from_str::<Fb>(&std::fs::read_to_string(path).ok()?).ok()? {
            Fb::Fb(fb) => Some(Self::Fb(FbProj::new(fb))),
            Fb::SFb(fb) => Some(Self::SFb(SFbProj::new(fb))),
        }
    }

    pub(crate) fn fb_state(&self) -> (f64, io::Fb) {
        match self {
            ProjSwitch::Fb(proj) => (proj.angle, io::Fb::Fb(proj.fb.clone())),
            ProjSwitch::SFb(proj) => (proj.angle, io::Fb::SFb(proj.fb.clone())),
        }
    }

    pub(crate) fn curve(&self) -> Curve {
        match self {
            ProjSwitch::Fb(proj) => Curve::P(proj.cache.curves.iter().map(|[.., c]| *c).collect()),
            ProjSwitch::SFb(proj) => Curve::S(proj.cache.curves.iter().map(|[.., c]| *c).collect()),
        }
    }

    impl_method! {
        fn show(self: &mut Self, ui: &mut Ui, pivot: &mut Pivot, cfg: &Cfg);
        fn plot(self: &Self, ui: &mut plot::PlotUi, ind: usize, id: usize);
        fn cache(self: &mut Self, res: usize);
        fn request_cache(self: &mut Self);
        fn name(self: &Self) -> String;
        fn preload(self: &mut Self);
        fn set_path(self: &mut Self, path: PathBuf);
        fn path(self: &Self) -> Option<&PathBuf>;
    }
}

type FbProj = ProjInner<efd::D2, NormFourBar>;
type SFbProj = ProjInner<efd::D3, SNormFourBar>;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct ProjInner<D, M>
where
    D: efd::EfdDim,
    M: Normalized<D>,
    M::De: CurveGen<D> + undo::IntoDelta,
{
    path: Option<PathBuf>,
    fb: M::De,
    angle: f64,
    hide: bool,
    #[serde(skip)]
    cache: Cache<D>,
    #[serde(skip)]
    undo: undo::Undo<<M::De as undo::IntoDelta>::Delta>,
}

impl<D, M> Default for ProjInner<D, M>
where
    D: efd::EfdDim,
    M: Normalized<D>,
    M::De: CurveGen<D> + undo::IntoDelta + Default,
{
    fn default() -> Self {
        Self {
            path: Default::default(),
            fb: Default::default(),
            angle: 0.,
            hide: false,
            cache: Default::default(),
            undo: Default::default(),
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

impl<D, M> ProjInner<D, M>
where
    D: efd::EfdDim,
    M: Normalized<D>,
    M::De: CurveGen<D>
        + undo::IntoDelta
        + undo::DeltaUi
        + undo::DeltaPlot<D>
        + Default
        + for<'a> Deserialize<'a>,
    efd::Coord<D>: Serialize,
    FourBarTy: for<'a> From<&'a M::De>,
{
    fn new(fb: M::De) -> Self {
        Self { fb, ..Self::default() }
    }

    fn new_with_path(path: Option<PathBuf>, fb: M::De) -> Self {
        Self { path, fb, ..Self::default() }
    }

    fn show(&mut self, ui: &mut Ui, pivot: &mut Pivot, cfg: &Cfg) {
        path_label(ui, "🖹", self.path.as_ref(), "Untitled");
        ui.label("Linkage type:");
        ui.label(FourBarTy::from(&self.fb).name());
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
        let get_curve = |pivot: Pivot, fb: &M::De, n: usize| -> Vec<_> {
            let curve = fb.curves(n).into_iter();
            match pivot {
                Pivot::Driver => curve.map(|[c, _, _]| c).collect(),
                Pivot::Follower => curve.map(|[_, c, _]| c).collect(),
                Pivot::Coupler => curve.map(|[_, _, c]| c).collect(),
            }
        };
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
                let t = csv::dump_csv(get_curve(*pivot, &self.fb, cfg.res)).unwrap();
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
                self.fb = M::normalize(&self.fb).denormalize();
                self.cache.changed = true;
            }
        });
        ui.separator();
        ui.heading("Parameters");
        let mut res = undo::DeltaUi::delta_ui(&mut self.fb, ui, cfg);
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

    fn cache(&mut self, res: usize) {
        if self.cache.changed {
            // Recalculation
            self.cache.changed = false;
            self.cache.joints = self.fb.pos(self.angle);
            self.cache.angle_bound = self.fb.angle_bound();
            self.cache.curves = self.fb.curves(res);
        }
    }

    fn plot(&self, ui: &mut plot::PlotUi, ind: usize, id: usize) {
        if !self.hide {
            let joints = self.cache.joints.as_ref();
            undo::DeltaPlot::delta_plot(&self.fb, ui, joints, &self.cache.curves, ind == id);
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
        if let Some(fb) = self
            .path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| ron::from_str(&s).ok())
        {
            self.fb = fb;
        }
    }

    fn set_path(&mut self, path: PathBuf) {
        self.path.replace(path);
    }

    fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }
}
