use self::impl_io::*;
use crate::csv::dump_csv;
use four_bar::{curve, plot, FourBar};

const FMT: &str = "Rusty Object Notation (RON)";
const EXT: &[&str] = &["ron"];
const CSV_FMT: &str = "Delimiter-Separated Values (CSV)";
const CSV_EXT: &[&str] = &["csv", "txt"];
const SVG_FMT: &str = "Scalable Vector Graphics (SVG)";
const SVG_EXT: &[&str] = &["svg"];

#[cfg(target_arch = "wasm32")]
mod impl_io {
    use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsValue};

    #[wasm_bindgen]
    extern "C" {
        fn open_file(ext: &str, done: JsValue, multiple: bool);
        fn save_file(s: &str, path: &str);
    }

    fn js_ext(ext: &[&str]) -> String {
        ext.iter()
            .map(|s| format!(".{s}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(super) fn open<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(String, String) + 'static,
    {
        let done = Closure::<dyn Fn(String, String)>::wrap(Box::new(done)).into_js_value();
        open_file(&js_ext(ext), done, true);
    }

    pub(super) fn open_single<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(String, String) + 'static,
    {
        open_file(&js_ext(ext), Closure::once_into_js(done), false);
    }

    pub(super) fn save_ask<C>(s: &str, file_name: &str, _fmt: &str, _ext: &[&str], done: C)
    where
        C: FnOnce(String) + 'static,
    {
        save(s, file_name);
        done(file_name.to_string());
    }

    pub(super) fn save(s: &str, path: &str) {
        save_file(s, path);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod impl_io {
    pub(super) fn open<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(String, String) + 'static,
    {
        if let Some(paths) = rfd::FileDialog::new().add_filter(fmt, ext).pick_files() {
            for path in paths {
                let s = std::fs::read_to_string(&path).unwrap_or_default();
                done(path.to_str().unwrap().to_string(), s);
            }
        }
    }

    pub(super) fn open_single<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(String, String) + 'static,
    {
        if let Some(path) = rfd::FileDialog::new().add_filter(fmt, ext).pick_file() {
            let s = std::fs::read_to_string(&path).unwrap_or_default();
            done(path.to_str().unwrap().to_string(), s);
        }
    }

    pub(super) fn save_ask<C>(s: &str, name: &str, fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(String) + 'static,
    {
        if let Some(file_name) = rfd::FileDialog::new()
            .set_file_name(name)
            .add_filter(fmt, ext)
            .save_file()
        {
            std::fs::write(&file_name, s).unwrap_or_default();
            done(file_name.to_str().unwrap().to_string());
        }
    }

    pub(super) fn save(s: &str, path: &str) {
        std::fs::write(path, s).unwrap_or_default();
    }
}

pub fn open_ron<C>(done: C)
where
    C: Fn(String, String) + 'static,
{
    open(FMT, EXT, done)
}

pub fn open_csv_single<C>(done: C)
where
    C: Fn(String, String) + 'static,
{
    open_single(CSV_FMT, CSV_EXT, done)
}

pub fn save_csv_ask<S>(curve: &[S])
where
    S: serde::Serialize + Clone,
{
    let s = dump_csv(curve).unwrap();
    save_ask(&s, "curve.csv", CSV_FMT, CSV_EXT, |_| ())
}

pub fn save_ron_ask<C>(fb: &FourBar, name: &str, done: C)
where
    C: FnOnce(String) + 'static,
{
    save_ask(&ron::to_string(fb).unwrap(), name, FMT, EXT, done)
}

pub fn save_ron(fb: &FourBar, path: &str) {
    save(&ron::to_string(fb).unwrap(), path)
}

pub fn save_history_ask(history: &[f64], name: &str) {
    let mut buf = String::new();
    let svg = plot::SVGBackend::with_string(&mut buf, (800, 600));
    plot::history(svg, history).unwrap();
    save_ask(&buf, name, SVG_FMT, SVG_EXT, |_| ())
}

pub fn save_curve_ask<F>(target: &[[f64; 2]], curve: &[[f64; 2]], fb: F, name: &str)
where
    F: Into<plot::FbOpt>,
{
    let mut buf = String::new();
    let svg = plot::SVGBackend::with_string(&mut buf, (800, 800));
    let curves = [("Target", target), ("Optimized", curve)];
    if !target.is_empty() && target.len() == curve.len() {
        let title = format!("Comparison (Error: {:.04})", curve::geo_err(target, curve));
        plot::curve(svg, &title, &curves, fb).unwrap();
    } else {
        plot::curve(svg, "Comparison", &curves, fb).unwrap();
    }
    save_ask(&buf, name, SVG_FMT, SVG_EXT, |_| ())
}
