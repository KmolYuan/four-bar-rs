use crate::dump_csv;
use four_bar::FourBar;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsValue};

#[macro_export]
macro_rules! ext {
    () => {
        "ron"
    };
}

const FMT: &str = "Rusty Object Notation";
const CSV_FMT: &str = "Delimiter-Separated Values";
const EXT: &[&str] = &[ext!()];
const CSV_EXT: &[&str] = &["csv", "txt"];

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    fn open_file(ext: &str, done: JsValue, multiple: bool);
    fn save_file(s: &str, path: &str);
    fn get_host() -> String;
    fn get_username() -> String;
    fn login(account: &str, body: &str, done: JsValue);
    fn logout(done: JsValue);
}

#[cfg(target_arch = "wasm32")]
fn js_ext(ext: &[&str]) -> String {
    ext.iter()
        .map(|s| format!(".{}", s))
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(Clone, Serialize, Deserialize)]
pub struct IoCtx {
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(serialize_with = "self::serde_agent::serialize")]
    #[serde(deserialize_with = "self::serde_agent::deserialize")]
    agent: ureq::Agent,
}

impl Default for IoCtx {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            agent: ureq::Agent::new(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl IoCtx {
    pub fn alert(s: &str) {
        alert(s);
    }

    fn open<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(String, String) + 'static,
    {
        let done = Closure::<dyn Fn(String, String)>::wrap(Box::new(done)).into_js_value();
        open_file(&js_ext(ext), done, true);
    }

    fn open_single<C>(_fmt: &str, ext: &[&str], done: C)
    where
        C: FnOnce(String, String) + 'static,
    {
        open_file(&js_ext(ext), Closure::once_into_js(done), false);
    }

    fn save_ask<C>(s: &str, file_name: &str, _fmt: &str, _ext: &[&str], done: C)
    where
        C: FnOnce(String) + 'static,
    {
        Self::save(s, file_name);
        done(file_name.to_string());
    }

    fn save(s: &str, path: &str) {
        save_file(s, path);
    }

    pub fn get_host() -> String {
        get_host()
    }

    pub fn get_username(&self, _url: &str) -> Option<String> {
        Some(get_username())
    }

    pub fn login<C>(&self, _url: &str, account: &str, body: &str, done: C)
    where
        C: FnOnce(bool) + 'static,
    {
        login(account, body, Closure::once_into_js(done));
    }

    pub fn logout<C>(&self, _url: &str, done: C)
    where
        C: FnOnce(bool) + 'static,
    {
        logout(Closure::once_into_js(done));
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl IoCtx {
    pub fn alert(s: &str) {
        rfd::MessageDialog::new()
            .set_title("Message")
            .set_description(s)
            .show();
    }

    fn open<C>(fmt: &str, ext: &[&str], done: C)
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

    fn open_single<C>(fmt: &str, ext: &[&str], done: C)
    where
        C: Fn(String, String) + 'static,
    {
        if let Some(path) = rfd::FileDialog::new().add_filter(fmt, ext).pick_file() {
            let s = std::fs::read_to_string(&path).unwrap_or_default();
            done(path.to_str().unwrap().to_string(), s);
        }
    }

    fn save_ask<C>(s: &str, name: &str, fmt: &str, ext: &[&str], done: C)
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

    fn save(s: &str, path: &str) {
        std::fs::write(path, s).unwrap_or_default();
    }

    pub fn get_host() -> String {
        "http://localhost:8080/".to_string()
    }

    pub fn get_username(&self, url: &str) -> Option<String> {
        if self.agent.get(url).call().is_err() {
            None
        } else if let Ok(uri) = url.parse::<actix_web::http::Uri>() {
            match self
                .agent
                .cookie_store()
                .get(uri.host().unwrap_or_default(), "/", "username")
            {
                Some(name) => Some(name.value().to_string()),
                None => Some(String::new()),
            }
        } else {
            None
        }
    }

    pub fn login<C>(&self, url: &str, account: &str, body: &str, done: C)
    where
        C: FnOnce(bool) + 'static,
    {
        let b = self
            .agent
            .post(&[url.trim_end_matches('/'), "login", account].join("/"))
            .set("content-type", "application/json")
            .send_bytes(body.as_bytes())
            .is_ok();
        done(b);
    }

    pub fn logout<C>(&self, url: &str, done: C)
    where
        C: FnOnce(bool) + 'static,
    {
        let b = self
            .agent
            .post(&[url.trim_end_matches('/'), "logout"].join("/"))
            .call()
            .is_ok();
        done(b);
    }
}

impl IoCtx {
    pub fn open_ron<C>(done: C)
    where
        C: Fn(String, String) + 'static,
    {
        Self::open(FMT, EXT, done)
    }

    pub fn open_csv_single<C>(done: C)
    where
        C: Fn(String, String) + 'static,
    {
        Self::open_single(CSV_FMT, CSV_EXT, done)
    }

    pub fn save_csv_ask<S>(curve: &[S])
    where
        S: Serialize + Clone,
    {
        let s = dump_csv(curve).unwrap();
        Self::save_ask(&s, "curve.csv", CSV_FMT, CSV_EXT, |_| ())
    }

    pub fn save_ron_ask<C>(four_bar: &FourBar, name: &str, done: C)
    where
        C: FnOnce(String) + 'static,
    {
        let s = ron::to_string(four_bar).unwrap();
        Self::save_ask(&s, name, FMT, EXT, done)
    }

    pub fn save_ron(four_bar: &FourBar, path: &str) {
        let s = ron::to_string(four_bar).unwrap();
        Self::save(&s, path)
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod serde_agent {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub(super) fn serialize<S>(agent: &ureq::Agent, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        agent.cookie_store().serialize(serializer)
    }

    pub(super) fn deserialize<'a, D>(deserializer: D) -> Result<ureq::Agent, D::Error>
    where
        D: Deserializer<'a>,
    {
        let cookies = Deserialize::deserialize(deserializer)?;
        Ok(ureq::AgentBuilder::new().cookie_store(cookies).build())
    }
}
