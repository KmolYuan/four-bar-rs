use self::impl_io::*;
use four_bar::{cb::FbCodebook, csv::dump_csv, plot2d, FourBar};
use std::path::{Path, PathBuf};

const FMT: &str = "Rusty Object Notation (RON)";
const EXT: &[&str] = &["ron"];
const CSV_FMT: &str = "Delimiter-Separated Values (CSV)";
const CSV_EXT: &[&str] = &["csv", "txt"];
const CB_FMT: &str = "Numpy Array Zip (NPZ)";
const CB_EXT: &[&str] = &["npz"];
const SVG_FMT: &str = "Scalable Vector Graphics (SVG)";
const SVG_EXT: &[&str] = &["svg"];

#[cfg(target_arch = "wasm32")]
mod impl_io {
    use std::path::{Path, PathBuf};
    use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsValue};

    #[wasm_bindgen]
    extern "C" {
        fn alert(s: &str);
        fn open_file(ext: &str, done: JsValue, multiple: bool);
        fn open_bfile(ext: &str, done: JsValue);
        fn save_file(s: &str, path: &str);
    }

    fn js_ext(ext: &[&str]) -> String {
        ext.iter()
            .map(|s| format!(".{s}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(super) fn alert_dialog(s: &str) {
        alert(s)
    }

    pub(super) fn open<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(PathBuf, String) + 'static,
    {
        let done = move |path, s| done(PathBuf::from(path), s);
        let done = Closure::<dyn Fn(String, String)>::wrap(Box::new(done)).into_js_value();
        open_file(&js_ext(ext), done, true);
    }

    pub(super) fn open_bin<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(Vec<u8>) + 'static,
    {
        let done = Closure::<dyn Fn(Vec<u8>)>::wrap(Box::new(done)).into_js_value();
        open_bfile(&js_ext(ext), done);
    }

    pub(super) fn open_single<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf, String) + 'static,
    {
        let done = |path: String, s| done(PathBuf::from(path), s);
        open_file(&js_ext(ext), Closure::once_into_js(done), false);
    }

    pub(super) fn save_ask<C>(s: &str, file_name: &str, _fmt: &str, _ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf) + 'static,
    {
        let file_name = PathBuf::from(file_name);
        save(s, &file_name);
        done(file_name);
    }

    pub(super) fn save(s: &str, path: &Path) {
        save_file(s, path.to_str().unwrap());
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod impl_io {
    use super::alert;
    use std::path::{Path, PathBuf};

    pub(super) fn alert_dialog(s: &str) {
        rfd::MessageDialog::new()
            .set_title("Error")
            .set_description(s)
            .set_level(rfd::MessageLevel::Error)
            .show();
    }

    pub(super) fn open<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(PathBuf, String) + 'static,
    {
        if let Some(paths) = rfd::FileDialog::new().add_filter(fmt, ext).pick_files() {
            for path in paths {
                alert(std::fs::read_to_string(&path), |s| {
                    done(path, s);
                });
            }
        }
    }

    pub(super) fn open_bin<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(Vec<u8>) + 'static,
    {
        if let Some(paths) = rfd::FileDialog::new().add_filter(fmt, ext).pick_files() {
            for path in paths {
                alert(std::fs::read(path), &done);
            }
        }
    }

    pub(super) fn open_single<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(PathBuf, String) + 'static,
    {
        if let Some(path) = rfd::FileDialog::new().add_filter(fmt, ext).pick_file() {
            alert(std::fs::read_to_string(&path), |s| done(path, s));
        }
    }

    pub(super) fn save_ask<C>(s: &str, name: &str, fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(PathBuf) + 'static,
    {
        if let Some(file_name) = rfd::FileDialog::new()
            .set_file_name(name)
            .add_filter(fmt, ext)
            .save_file()
        {
            alert(std::fs::write(&file_name, s), |_| done(file_name));
        }
    }

    pub(super) fn save(s: &str, path: &Path) {
        alert(std::fs::write(path, s), |_| ());
    }
}

pub(crate) fn alert<T, E, C>(r: Result<T, E>, done: C)
where
    E: std::error::Error,
    C: FnOnce(T),
{
    match r {
        Ok(t) => done(t),
        Err(e) => alert_dialog(&e.to_string()),
    }
}

pub(crate) fn open_ron<C>(done: C)
where
    C: Fn(PathBuf, FourBar) + 'static,
{
    let done = move |path, s: String| alert(ron::from_str(&s), |fb| done(path, fb));
    open(FMT, EXT, done);
}

pub(crate) fn open_csv_single<C>(done: C)
where
    C: Fn(PathBuf, String) + 'static,
{
    open_single(CSV_FMT, CSV_EXT, done);
}

pub(crate) fn open_cb<C>(done: C)
where
    C: Fn(FbCodebook) + 'static,
{
    let done = move |b| alert(FbCodebook::read(std::io::Cursor::new(b)), &done);
    open_bin(CB_FMT, CB_EXT, done);
}

pub(crate) fn save_csv_ask<S>(curve: &[S])
where
    S: serde::Serialize + Clone,
{
    let s = dump_csv(curve).unwrap();
    save_ask(&s, "curve.csv", CSV_FMT, CSV_EXT, |_| ());
}

pub(crate) fn save_ron_ask<C>(fb: &FourBar, name: &str, done: C)
where
    C: FnOnce(PathBuf) + 'static,
{
    save_ask(&ron::to_string(fb).unwrap(), name, FMT, EXT, done);
}

pub(crate) fn save_ron(fb: &FourBar, path: &Path) {
    save(&ron::to_string(fb).unwrap(), path);
}

pub(crate) fn save_history_ask(history: &[f64], name: &str) {
    let mut buf = String::new();
    let svg = plot2d::SVGBackend::with_string(&mut buf, (800, 600));
    plot2d::history(svg, history).unwrap();
    save_ask(&buf, name, SVG_FMT, SVG_EXT, |_| ());
}

pub(crate) fn save_curve_ask<'a, 'b, C, O>(curves: C, opt: O, name: &str)
where
    C: IntoIterator<Item = (&'b str, &'b [[f64; 2]])>,
    plot2d::Opt<'a, 'b>: From<O>,
{
    let mut buf = String::new();
    let svg = plot2d::SVGBackend::with_string(&mut buf, (800, 800));
    plot2d::plot(svg, curves, opt).unwrap();
    save_ask(&buf, name, SVG_FMT, SVG_EXT, |_| ());
}
