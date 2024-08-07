use self::impl_io::*;
use four_bar::*;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

pub(crate) type Cache<T> = std::rc::Rc<std::cell::RefCell<Option<T>>>;
const FMT: &str = "Rusty Object Notation (RON)";
const EXT: &[&str] = &["ron"];
const CSV_FMT: &str = "Delimiter-Separated Values (CSV)";
const CSV_EXT: &[&str] = &["csv", "txt"];
const ATLAS_FMT: &str = "Numpy Array Zip (NPZ)";
const ATLAS_EXT: &[&str] = &["npz"];
const SVG_FMT: &str = "Scalable Vector Graphics (SVG)";
const SVG_EXT: &[&str] = &["svg"];
const GIF_FMT: &str = "Graphics Interchange Format (GIF)";
const GIF_EXT: &[&str] = &["gif"];
const IMG_FMT: &str = "Supported Image Format (PNG & JPEG)";
const IMG_EXT: &[&str] = &["png", "jpg", "jpeg"];

// A powerful macro for alerting the user in the GUI
macro_rules! alert {
    // end: do nothing
    (@) => { |_| () };
    // for closures
    (@($title1:literal, |$x:pat_param| $expr1:expr) $(, ($title:literal, $($expr:tt)+))* $(,)?) => {
        |$x| $expr1.alert_then($title1, alert!(@$(($title, $($expr)+)),*))
    };
    // for functions from path
    (@($title1:literal, $expr1:path) $(, ($title:literal, $($expr:tt)+))* $(,)?) => {
        |x| $expr1(x).alert_then($title1, alert!(@$(($title, $($expr)+)),*))
    };
    // (pub) single pair
    ($title:literal, $expr:expr) => {
        $crate::io::alert!(($title, $expr))
    };
    // (pub) multiple pairs
    (($title1:literal, $expr1:expr) $(, ($title:literal, $($expr:tt)+))* $(,)?) => {{
        #[allow(unused_imports)]
        use $crate::io::{Alert as _, alert};
        $expr1.alert_then($title1, alert!(@$(($title, $($expr)+)),*))
    }};
}
pub(crate) use alert;

#[cfg(target_arch = "wasm32")]
mod impl_io {
    use std::{
        io::Cursor,
        path::{Path, PathBuf},
    };
    use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsValue};

    #[wasm_bindgen]
    extern "C" {
        fn open_file(ext: &str, done: JsValue, is_multiple: bool, is_bin: bool);
        fn save_file(s: &[u8], path: &str);
    }

    fn js_ext(ext: &[&str]) -> String {
        ext.iter()
            .map(|s| format!(".{s}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(super) fn open<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(PathBuf, Cursor<String>) + 'static,
    {
        let done = move |path, s| done(PathBuf::from(path), Cursor::new(s));
        let done = Closure::<dyn Fn(String, String)>::wrap(Box::new(done)).into_js_value();
        open_file(&js_ext(ext), done, true, false);
    }

    pub(super) fn open_bin<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(Cursor<Vec<u8>>) + 'static,
    {
        let done = move |buf| done(Cursor::new(buf));
        let done = Closure::<dyn Fn(Vec<u8>)>::wrap(Box::new(done)).into_js_value();
        open_file(&js_ext(ext), done, true, true);
    }

    pub(super) fn open_single<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf, Cursor<String>) + 'static,
    {
        let done = |path: String, s| done(PathBuf::from(path), Cursor::new(s));
        open_file(&js_ext(ext), Closure::once_into_js(done), false, false);
    }

    pub(super) fn open_bin_single<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf, Cursor<Vec<u8>>) + 'static,
    {
        let done = move |path: String, buf| done(PathBuf::from(path), Cursor::new(buf));
        open_file(&js_ext(ext), Closure::once_into_js(done), false, true);
    }

    pub(super) fn save_ask<W, E, C>(name: &str, _fmt: &str, _ext: &[&str], write: W, done: C)
    where
        W: FnOnce(Cursor<&mut Vec<u8>>) -> E,
        E: super::Alert,
        C: FnOnce(PathBuf),
    {
        let path = PathBuf::from(name);
        save(&path, write);
        done(path);
    }

    pub(super) fn save<W, E>(name: &Path, write: W)
    where
        W: FnOnce(Cursor<&mut Vec<u8>>) -> E,
        E: super::Alert,
    {
        let mut buf = Vec::new();
        alert!("Write File", write(Cursor::new(&mut buf)));
        save_file(&buf, name.as_os_str().to_str().unwrap());
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod impl_io {
    use std::{
        fs::File,
        path::{Path, PathBuf},
    };

    pub(super) fn open<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(PathBuf, File) + 'static,
    {
        if let Some(paths) = rfd::FileDialog::new().add_filter(fmt, ext).pick_files() {
            for path in paths {
                alert!(("Open File", File::open(&path)), ("*", |r| done(path, r)));
            }
        }
    }

    pub(super) fn open_bin<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(File) + 'static,
    {
        if let Some(paths) = rfd::FileDialog::new().add_filter(fmt, ext).pick_files() {
            for path in paths {
                alert!(("Open File", File::open(path)), ("*", done));
            }
        }
    }

    pub(super) fn open_single<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf, File) + 'static,
    {
        if let Some(path) = rfd::FileDialog::new().add_filter(fmt, ext).pick_file() {
            alert!(("Open File", File::open(&path)), ("*", |s| done(path, s)));
        }
    }

    pub(super) fn open_bin_single<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf, File) + 'static,
    {
        if let Some(path) = rfd::FileDialog::new().add_filter(fmt, ext).pick_file() {
            alert!(("Open File", File::open(&path)), ("*", |s| done(path, s)));
        }
    }

    pub(super) fn save_ask<W, E, C>(name: &str, fmt: &str, ext: &[&str], write: W, done: C)
    where
        W: FnOnce(File) -> E,
        E: super::Alert,
        C: FnOnce(PathBuf),
    {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(name)
            .add_filter(fmt, ext)
            .save_file()
        {
            alert!(
                ("Save File", File::create(&path)),
                ("Write File", write),
                ("*", |_| done(path))
            );
        }
    }

    pub(super) fn save<W, E>(path: &Path, write: W)
    where
        W: FnOnce(File) -> E,
        E: super::Alert,
    {
        alert!(("Save File", File::create(path)), ("Write File", write));
    }
}

pub(crate) trait Alert: Sized {
    type Output;
    fn alert_then<C>(self, title: &'static str, done: C)
    where
        C: FnOnce(Self::Output);
}

impl Alert for () {
    type Output = ();
    #[inline]
    fn alert_then<C>(self, _title: &'static str, done: C)
    where
        C: FnOnce(Self::Output),
    {
        done(());
    }
}

impl<T, E: std::error::Error> Alert for Result<T, E> {
    type Output = T;
    fn alert_then<C>(self, title: &'static str, done: C)
    where
        C: FnOnce(Self::Output),
    {
        match self {
            Ok(t) => done(t),
            Err(e) => push_alert(title, e.to_string()),
        }
    }
}

impl<T> Alert for Option<T> {
    type Output = T;
    fn alert_then<C>(self, title: &'static str, done: C)
    where
        C: FnOnce(Self::Output),
    {
        match self {
            Some(t) => done(t),
            None => push_alert(title, "Operation failed."),
        }
    }
}

static ERR_MSG: std::sync::Mutex<Option<(Cow<'static, str>, Cow<'static, str>)>> =
    std::sync::Mutex::new(None);

#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
pub(crate) fn show_err_msg(parent: &eframe::Frame) {
    let Some((title, msg)) = ERR_MSG.lock().unwrap().take() else {
        return;
    };
    macro_rules! msg {
        ($ty:ident $(, $parent:ident)?) => {
            rfd::$ty::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title(&*title)
                .set_description(&*msg)
                $(.set_parent($parent))?
                .show()
        };
    }
    #[cfg(not(target_arch = "wasm32"))]
    msg!(MessageDialog, parent);
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move { _ = msg!(AsyncMessageDialog).await });
}

fn push_alert<S1, S2>(title: S1, msg: S2)
where
    S1: Into<Cow<'static, str>>,
    S2: Into<Cow<'static, str>>,
{
    *ERR_MSG.lock().unwrap() = Some((title.into(), msg.into()));
}

pub(crate) fn open_ron_single<S, C>(done: C)
where
    S: serde::de::DeserializeOwned,
    C: FnOnce(PathBuf, S) + 'static,
{
    open_single(FMT, EXT, move |path, r| {
        alert!(
            ("Parse File", ron::de::from_reader(r)),
            ("*", |s| done(path, s))
        );
    });
}

pub(crate) fn open_ron<S, C>(done: C)
where
    S: serde::de::DeserializeOwned,
    C: Fn(PathBuf, S) + 'static,
{
    open(FMT, EXT, move |path, r| {
        alert!(
            ("Parse File", ron::de::from_reader(r)),
            ("*", |s| done(path, s))
        );
    });
}

pub(crate) fn open_csv_single<C>(done: C)
where
    C: FnOnce(PathBuf, Curve) + 'static,
{
    open_single(CSV_FMT, CSV_EXT, move |p, r| {
        alert!(
            ("Parse File", Curve::from_csv_reader(r)),
            ("*", |d| done(p, d))
        );
    });
}

pub(crate) fn open_csv<C>(done: C)
where
    C: Fn(PathBuf, Curve) + 'static,
{
    open(CSV_FMT, CSV_EXT, move |p, r| {
        alert!(
            ("Parse File", Curve::from_csv_reader(r)),
            ("*", |d| done(p, d))
        );
    });
}

pub(crate) fn open_atlas<C>(done: C)
where
    C: Fn(Atlas) + 'static,
{
    let done = move |b| alert!(("Parse File", Atlas::from_reader(b)), ("*", done));
    open_bin(ATLAS_FMT, ATLAS_EXT, done);
}

pub(crate) fn open_img<C>(done: C)
where
    C: FnOnce(PathBuf, ColorImage) + 'static,
{
    open_bin_single(IMG_FMT, IMG_EXT, move |path, buf| {
        alert!(
            ("Load Image", load_img(buf)),
            ("Parse File", |img| done(path, img))
        );
    });
}

pub(crate) fn save_csv_ask<S>(c: &[S])
where
    S: serde::Serialize,
{
    save_ask(
        "curve.csv",
        CSV_FMT,
        CSV_EXT,
        |w| csv::to_writer(w, c),
        |_| (),
    );
}

pub(crate) fn save_atlas_ask<M, const N: usize, const D: usize>(atlas: &atlas::Atlas<M, N, D>) {
    save_ask(
        "atlas.npz",
        ATLAS_FMT,
        ATLAS_EXT,
        |w| atlas.write(w),
        |_| (),
    );
}

fn write_ron<W, S>(mut w: W, s: &S) -> Result<(), ron::Error>
where
    W: std::io::Write,
    S: serde::Serialize,
{
    write!(w, "{}", ron::ser::to_string_pretty(s, Default::default())?).map_err(|e| e.into())
}

pub(crate) fn save_ron_ask<S, C>(s: &S, name: &str, done: C)
where
    S: serde::Serialize,
    C: FnOnce(PathBuf) + 'static,
{
    save_ask(name, FMT, EXT, |w| write_ron(w, s), done);
}

pub(crate) fn save_ron<S>(s: &S, path: &Path)
where
    S: serde::Serialize,
{
    save(path, |w| write_ron(w, s));
}

pub(crate) fn save_svg_ask(buf: &str, name: &str) {
    use std::io::Write as _;
    save_ask(
        name,
        SVG_FMT,
        SVG_EXT,
        |mut w| w.write_all(buf.as_bytes()),
        |_| (),
    );
}

pub(crate) fn save_gif_ask(buf: Vec<u8>, name: &str) {
    use std::io::Write as _;
    save_ask(name, GIF_FMT, GIF_EXT, |mut w| w.write_all(&buf), |_| ());
}

pub(crate) fn save_history_ask(history: &[f64], name: &str) {
    let mut buf = String::new();
    let svg = plot::SVGBackend::with_string(&mut buf, (800, 600));
    plot::fb::history(svg, history).unwrap();
    save_svg_ask(&buf, name);
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(untagged)]
pub(crate) enum Fb {
    P(FourBar),
    M(MFourBar),
    S(SFourBar),
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(crate) enum Curve {
    P(Vec<[f64; 2]>),
    M(Vec<([f64; 2], [f64; 2])>),
    S(Vec<[f64; 3]>),
}

impl Default for Curve {
    fn default() -> Self {
        Self::P(Vec::new())
    }
}

impl Curve {
    pub(crate) fn from_csv_reader<R>(mut r: R) -> Result<Self, csv::Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        // Please be aware of the order of the array size,
        // it should be in descending order to avoid ambiguity.
        (csv::from_reader(&mut r).map(Self::M)) // 4
            .or_else(|_| {
                r.rewind()?;
                csv::from_reader(&mut r).map(Self::S) // 3
            })
            .or_else(|_| {
                r.rewind()?;
                csv::from_reader(r).map(Self::P) // 2
            })
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Curve::P(c) => c.len(),
            Curve::M(c) => c.len(),
            Curve::S(c) => c.len(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn convert_to_planar(&mut self) {
        match self {
            Self::S(c) => *self = Self::P(c.iter().map(|&[x, y, _]| [x, y]).collect()),
            Self::M(c) => *self = Self::P(c.iter().map(|&([x, y], _)| [x, y]).collect()),
            Self::P(_) => (),
        }
    }

    pub(crate) fn convert_to_motion(&mut self) {
        match self {
            Self::P(c) => *self = Self::M(c.iter().map(|&[x, y]| ([x, y], [0., 0.])).collect()),
            Self::S(c) => *self = Self::M(c.iter().map(|&[x, y, _]| ([x, y], [0., 0.])).collect()),
            Self::M(_) => (),
        }
    }

    pub(crate) fn convert_to_spatial(&mut self) {
        match self {
            Self::P(c) => *self = Self::S(c.iter().map(|&[x, y]| [x, y, 0.]).collect()),
            Self::M(c) => *self = Self::S(c.iter().map(|&([x, y], [_, _])| [x, y, 0.]).collect()),
            Self::S(_) => (),
        }
    }
}

pub(crate) enum Atlas {
    P(atlas::FbAtlas),
    S(atlas::SFbAtlas),
}

impl Atlas {
    pub(crate) fn from_reader<R>(mut r: R) -> Result<Self, atlas::ReadNpzError>
    where
        R: std::io::Read + std::io::Seek,
    {
        if let Ok(atlas) = atlas::SFbAtlas::read(&mut r) {
            Ok(Self::S(atlas))
        } else {
            r.rewind().map_err(|e| atlas::ReadNpzError::Zip(e.into()))?;
            Ok(Self::P(atlas::FbAtlas::read(r)?))
        }
    }
}

#[derive(Default)]
pub(crate) struct AtlasPool {
    fb: atlas::FbAtlas,
    sfb: atlas::SFbAtlas,
}

impl From<Atlas> for AtlasPool {
    fn from(atlas: Atlas) -> Self {
        match atlas {
            Atlas::P(atlas) => Self { fb: atlas, sfb: Default::default() },
            Atlas::S(atlas) => Self { sfb: atlas, fb: Default::default() },
        }
    }
}

impl AtlasPool {
    pub(crate) fn merge_inplace(&mut self, rhs: Self) {
        self.fb
            .merge_inplace(rhs.fb)
            .unwrap_or_else(|_| unreachable!());
        self.sfb
            .merge_inplace(rhs.sfb)
            .unwrap_or_else(|_| unreachable!());
    }

    pub(crate) fn merge_atlas_inplace(&mut self, atlas: Atlas) {
        match atlas {
            Atlas::P(atlas) => (self.fb)
                .merge_inplace(atlas)
                .unwrap_or_else(|_| unreachable!()),
            Atlas::S(atlas) => (self.sfb)
                .merge_inplace(atlas)
                .unwrap_or_else(|_| unreachable!()),
        }
    }

    pub(crate) fn as_fb(&self) -> &atlas::FbAtlas {
        &self.fb
    }

    pub(crate) fn as_sfb(&self) -> &atlas::SFbAtlas {
        &self.sfb
    }

    pub(crate) fn as_fb_mut(&mut self) -> &mut atlas::FbAtlas {
        &mut self.fb
    }

    pub(crate) fn as_sfb_mut(&mut self) -> &mut atlas::SFbAtlas {
        &mut self.sfb
    }
}

impl FromIterator<Atlas> for AtlasPool {
    fn from_iter<T: IntoIterator<Item = Atlas>>(iter: T) -> Self {
        let mut pool = Self::default();
        iter.into_iter()
            .for_each(|atlas| pool.merge_atlas_inplace(atlas));
        pool
    }
}

use eframe::egui::ColorImage;

pub(crate) fn load_img<R>(r: R) -> Result<ColorImage, image::ImageError>
where
    R: std::io::Read + std::io::Seek,
{
    let img = image::ImageReader::new(std::io::BufReader::new(r))
        .with_guessed_format()?
        .decode()?
        .to_rgba8();
    let size = [img.width(), img.height()].map(|s| s as _);
    Ok(ColorImage::from_rgba_unmultiplied(size, img.as_raw()))
}
