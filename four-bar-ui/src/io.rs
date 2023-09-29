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
const CB_FMT: &str = "Numpy Array Zip (NPZ)";
const CB_EXT: &[&str] = &["npz"];
const SVG_FMT: &str = "Scalable Vector Graphics (SVG)";
const SVG_EXT: &[&str] = &["svg"];
const IMG_FMT: &str = "Supported Image Format (PNG & JPEG)";
const IMG_EXT: &[&str] = &["png", "jpg", "jpeg"];

#[cfg(target_arch = "wasm32")]
mod impl_io {
    use super::Alert as _;
    use std::path::{Path, PathBuf};
    use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsValue};

    #[wasm_bindgen]
    extern "C" {
        fn open_file(ext: &str, done: JsValue, is_multiple: bool, is_bin: bool);
        fn save_file(s: &str, path: &str);
        #[wasm_bindgen(js_name = save_file)]
        fn save_bin(s: &[u8], path: &str);
    }

    fn js_ext(ext: &[&str]) -> String {
        ext.iter()
            .map(|s| format!(".{s}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(super) fn open<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(PathBuf, std::io::Cursor<String>) + 'static,
    {
        let done = move |path, s| done(PathBuf::from(path), std::io::Cursor::new(s));
        let done = Closure::<dyn Fn(String, String)>::wrap(Box::new(done)).into_js_value();
        open_file(&js_ext(ext), done, true, false);
    }

    pub(super) fn open_bin<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(std::io::Cursor<Vec<u8>>) + 'static,
    {
        let done = move |buf| done(std::io::Cursor::new(buf));
        let done = Closure::<dyn Fn(Vec<u8>)>::wrap(Box::new(done)).into_js_value();
        open_file(&js_ext(ext), done, true, true);
    }

    pub(super) fn open_single<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf, std::io::Cursor<String>) + 'static,
    {
        let done = |path: String, s| done(PathBuf::from(path), std::io::Cursor::new(s));
        open_file(&js_ext(ext), Closure::once_into_js(done), false, false);
    }

    pub(super) fn open_bin_single<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf, std::io::Cursor<Vec<u8>>) + 'static,
    {
        let done = move |path: String, buf| done(PathBuf::from(path), std::io::Cursor::new(buf));
        open_file(&js_ext(ext), Closure::once_into_js(done), false, true);
    }

    pub(super) fn save_ask<C>(s: &str, name: &str, _fmt: &str, _ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf) + 'static,
    {
        let name = PathBuf::from(name);
        save(s, &name);
        done(name);
    }

    pub(super) fn save(s: &str, name: &Path) {
        save_file(s, name.to_str().unwrap());
    }

    pub(super) fn save_bin_ask<C, E>(name: &str, _fmt: &str, _ext: &[&str], write: C)
    where
        C: FnOnce(std::io::Cursor<&mut [u8]>) -> Result<(), E>,
        E: std::error::Error,
    {
        let mut buf = Vec::new();
        write(std::io::Cursor::new(&mut buf)).alert("Save File");
        save_bin(&buf, name);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod impl_io {
    use super::Alert as _;
    use std::path::{Path, PathBuf};

    pub(super) fn open<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(PathBuf, std::fs::File) + 'static,
    {
        if let Some(paths) = rfd::FileDialog::new().add_filter(fmt, ext).pick_files() {
            for path in paths {
                std::fs::File::open(&path).alert_then("Open File", |r| done(path, r));
            }
        }
    }

    pub(super) fn open_bin<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(std::fs::File) + 'static,
    {
        if let Some(paths) = rfd::FileDialog::new().add_filter(fmt, ext).pick_files() {
            paths
                .into_iter()
                .for_each(|path| std::fs::File::open(path).alert_then("Open File", &done));
        }
    }

    pub(super) fn open_single<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf, std::fs::File) + 'static,
    {
        if let Some(path) = rfd::FileDialog::new().add_filter(fmt, ext).pick_file() {
            std::fs::File::open(&path).alert_then("Open File", |s| done(path, s));
        }
    }

    pub(super) fn open_bin_single<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf, std::fs::File) + 'static,
    {
        if let Some(path) = rfd::FileDialog::new().add_filter(fmt, ext).pick_file() {
            std::fs::File::open(&path).alert_then("Open File", |s| done(path, s));
        }
    }

    pub(super) fn save_ask<C>(s: &str, name: &str, fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf) + 'static,
    {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(name)
            .add_filter(fmt, ext)
            .save_file()
        {
            std::fs::write(&path, s).alert_then("Save File", |_| done(path));
        }
    }

    pub(super) fn save(s: &str, path: &Path) {
        std::fs::write(path, s).alert("Save File");
    }

    pub(super) fn save_bin_ask<C, E>(name: &str, fmt: &str, ext: &[&str], write: C)
    where
        C: FnOnce(std::fs::File) -> Result<(), E>,
        E: std::error::Error,
    {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(name)
            .add_filter(fmt, ext)
            .save_file()
        {
            std::fs::File::create(path).alert_then("Create File", |f| write(f).alert("Save File"));
        }
    }
}

pub(crate) trait Alert: Sized {
    type Output;

    fn alert_then<C>(self, title: &'static str, done: C)
    where
        C: FnOnce(Self::Output);

    fn alert(self, title: &'static str) {
        self.alert_then(title, |_| ());
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
            Err(e) => alert(title, format!("{e}")),
        }
    }
}

impl<T> Alert for Option<T> {
    type Output = T;

    fn alert_then<C>(self, msg: &'static str, done: C)
    where
        C: FnOnce(Self::Output),
    {
        match self {
            Some(t) => done(t),
            None => alert("", msg),
        }
    }
}

static ERR_MSG: std::sync::Mutex<Option<(Cow<'static, str>, Cow<'static, str>)>> =
    std::sync::Mutex::new(None);

#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
pub(crate) fn push_err_msg(parent: &eframe::Frame) {
    if let Some((title, msg)) = ERR_MSG.lock().unwrap().take() {
        macro_rules! msg {
            ($ty:ident $(, $parent: ident)?) => {
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
}

pub(crate) fn alert<S1, S2>(title: S1, msg: S2)
where
    S1: Into<Cow<'static, str>>,
    S2: Into<Cow<'static, str>>,
{
    ERR_MSG.lock().unwrap().replace((title.into(), msg.into()));
}

pub(crate) fn open_ron_single<C>(done: C)
where
    C: FnOnce(PathBuf, Fb) + 'static,
{
    let done = move |path, r| ron::de::from_reader(r).alert_then("Parse File", |fb| done(path, fb));
    open_single(FMT, EXT, done);
}

pub(crate) fn open_ron<C>(done: C)
where
    C: Fn(PathBuf, Fb) + 'static,
{
    let done = move |path, r| ron::de::from_reader(r).alert_then("Parse File", |fb| done(path, fb));
    open(FMT, EXT, done);
}

pub(crate) fn open_csv_single<C>(done: C)
where
    C: FnOnce(PathBuf, Curve) + 'static,
{
    open_single(CSV_FMT, CSV_EXT, move |p, r| {
        Curve::from_reader(r).alert_then("Parse File", |d| done(p, d));
    });
}

pub(crate) fn open_csv<C>(done: C)
where
    C: Fn(PathBuf, Curve) + 'static,
{
    open(CSV_FMT, CSV_EXT, move |p, r| {
        Curve::from_reader(r).alert_then("Parse File", |d| done(p, d));
    });
}

pub(crate) fn open_cb<C>(done: C)
where
    C: Fn(Cb) + 'static,
{
    let done = move |b| Cb::from_reader(b).alert_then("Parse File", &done);
    open_bin(CB_FMT, CB_EXT, done);
}

pub(crate) fn open_img<C>(done: C)
where
    C: FnOnce(PathBuf, ColorImage) + 'static,
{
    let done = move |path, buf| load_img(buf).alert_then("Parse File", |img| done(path, img));
    open_bin_single(IMG_FMT, IMG_EXT, done);
}

pub(crate) fn save_csv_ask<S>(curve: &[S])
where
    S: serde::Serialize + Clone,
{
    let s = csv::dump_csv(curve).unwrap();
    save_ask(&s, "curve.csv", CSV_FMT, CSV_EXT, |_| ());
}

pub(crate) fn save_cb_ask<C, D>(cb: &cb::Codebook<C, D>)
where
    C: cb::Code<D> + Send,
    D: efd::EfdDim,
{
    save_bin_ask("cb.npz", CB_FMT, CB_EXT, |w| cb.write(w));
}

pub(crate) fn save_ron_ask<S, C>(fb: &S, name: &str, done: C)
where
    S: serde::Serialize,
    C: FnOnce(PathBuf) + 'static,
{
    save_ask(&ron_string(fb), name, FMT, EXT, done);
}

pub(crate) fn save_ron<S>(fb: &S, path: &Path)
where
    S: serde::Serialize,
{
    save(&ron_string(fb), path);
}

pub(crate) fn save_svg_ask(buf: &str, name: &str) {
    save_ask(buf, name, SVG_FMT, SVG_EXT, |_| ());
}

pub(crate) fn save_history_ask(history: &[f64], name: &str) {
    let mut buf = String::new();
    let svg = plot::SVGBackend::with_string(&mut buf, (800, 600));
    plot2d::history(svg, history).unwrap();
    save_ask(&buf, name, SVG_FMT, SVG_EXT, |_| ());
}

pub(crate) fn ron_string<S>(value: &S) -> String
where
    S: serde::Serialize,
{
    // Use default options to serialize the data
    ron::ser::to_string_pretty(value, Default::default()).unwrap()
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(untagged)]
pub(crate) enum Fb {
    Fb(FourBar),
    SFb(SFourBar),
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(crate) enum Curve {
    P(Vec<[f64; 2]>),
    S(Vec<[f64; 3]>),
}

impl Default for Curve {
    fn default() -> Self {
        Self::P(Vec::new())
    }
}

impl Curve {
    pub(crate) fn from_reader<R>(mut r: R) -> Result<Self, csv::Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        if let Ok(c) = csv::parse_csv(&mut r) {
            Ok(Self::S(c))
        } else {
            r.rewind()?;
            Ok(Self::P(csv::parse_csv(r)?))
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Curve::P(c) => c.len(),
            Curve::S(c) => c.len(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn is_planar(&self) -> bool {
        matches!(self, Self::P(_))
    }

    pub(crate) fn convert_to_planar(&mut self) {
        if let Self::S(c) = self {
            let c = c.iter().map(|&[x, y, _]| [x, y]).collect();
            *self = Self::P(c);
        }
    }

    pub(crate) fn convert_to_spatial(&mut self) {
        if let Self::P(c) = self {
            let c = c.iter().map(|&[x, y]| [x, y, 0.]).collect();
            *self = Self::S(c);
        }
    }
}

pub(crate) enum Cb {
    P(cb::FbCodebook),
    S(cb::SFbCodebook),
}

impl Cb {
    pub(crate) fn from_reader<R>(mut r: R) -> Result<Self, cb::ReadNpzError>
    where
        R: std::io::Read + std::io::Seek,
    {
        if let Ok(cb) = cb::FbCodebook::read(&mut r) {
            Ok(Self::P(cb))
        } else {
            r.rewind().map_err(|e| cb::ReadNpzError::Zip(e.into()))?;
            Ok(Self::S(cb::SFbCodebook::read(r)?))
        }
    }
}

#[derive(Default)]
pub(crate) struct CbPool {
    fb: cb::FbCodebook,
    sfb: cb::SFbCodebook,
}

impl CbPool {
    pub(crate) fn merge_inplace(&mut self, cb: Cb) -> Result<(), mh::ndarray::ShapeError> {
        match cb {
            Cb::P(cb) => self.fb.merge_inplace(&cb),
            Cb::S(cb) => self.sfb.merge_inplace(&cb),
        }
    }

    pub(crate) fn as_fb(&self) -> &cb::FbCodebook {
        &self.fb
    }

    pub(crate) fn as_sfb(&self) -> &cb::SFbCodebook {
        &self.sfb
    }

    pub(crate) fn as_fb_mut(&mut self) -> &mut cb::FbCodebook {
        &mut self.fb
    }

    pub(crate) fn as_sfb_mut(&mut self) -> &mut cb::SFbCodebook {
        &mut self.sfb
    }
}

impl FromIterator<Cb> for CbPool {
    fn from_iter<T: IntoIterator<Item = Cb>>(iter: T) -> Self {
        let mut pool = Self::default();
        iter.into_iter().for_each(|cb| _ = pool.merge_inplace(cb));
        pool
    }
}

use eframe::egui::ColorImage;

pub(crate) fn load_img<R>(r: R) -> Result<ColorImage, image::ImageError>
where
    R: std::io::Read + std::io::Seek,
{
    let img = image::io::Reader::new(std::io::BufReader::new(r))
        .with_guessed_format()?
        .decode()?
        .to_rgba8();
    let size = [img.width(), img.height()].map(|s| s as _);
    Ok(ColorImage::from_rgba_unmultiplied(size, img.as_raw()))
}
