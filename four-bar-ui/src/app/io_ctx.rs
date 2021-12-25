use crate::app::Atomic;
#[cfg(target_arch = "wasm32")]
use {
    js_sys::{Array, JsString},
    wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsValue},
};
#[cfg(not(target_arch = "wasm32"))]
use {
    rfd::{FileDialog, MessageDialog},
    std::fs::{read_to_string, write},
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    fn save_file(s: &str, file_name: &str);
    fn load_file(buf: Array, format: &str);
    fn login(account: &str, body: &str, done: JsValue);
}

#[derive(Clone)]
pub(crate) struct IoCtx {
    #[cfg(target_arch = "wasm32")]
    buf: Array,
    #[cfg(not(target_arch = "wasm32"))]
    agent: ureq::Agent,
}

impl Default for IoCtx {
    fn default() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            buf: Array::new(),
            #[cfg(not(target_arch = "wasm32"))]
            agent: ureq::Agent::new(),
        }
    }
}

impl IoCtx {
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn open(&self, ext: &[&str]) {
        let format = ext
            .iter()
            .map(|s| format!(".{}", s))
            .collect::<Vec<_>>()
            .join(",");
        load_file(self.buf.clone(), &format);
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn open_result(&self) -> Option<String> {
        if self.buf.length() > 0 {
            Some(String::from(JsString::from(self.buf.pop())))
        } else {
            None
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn open(&self, fmt: &str, ext: &[&str]) -> Option<String> {
        if let Some(path) = FileDialog::new().add_filter(fmt, ext).pick_file() {
            read_to_string(path).ok()
        } else {
            None
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn save(&self, s: &str, file_name: &str) {
        save_file(s, file_name);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn save(&self, s: &str, name: &str, fmt: &str, ext: &[&str]) {
        if let Some(file_name) = rfd::FileDialog::new()
            .set_file_name(name)
            .add_filter(fmt, ext)
            .save_file()
        {
            write(file_name, s).unwrap_or_default();
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn alert(s: &str) {
        alert(s);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn alert(s: &str) {
        MessageDialog::new()
            .set_title("Message")
            .set_description(s)
            .show();
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn login(&self, _url: &str, account: &str, body: &str, state: Atomic<bool>) {
        let done = Closure::once_into_js(move |b| state.store(b));
        login(account, body, done);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn login(&self, url: &str, account: &str, body: &str, state: Atomic<bool>) {
        if self
            .agent
            .post(&[url.trim_end_matches('/'), "login", account].join("/"))
            .set("content-type", "application/json")
            .send_bytes(body.as_bytes())
            .is_ok()
        {
            Self::alert("Login successfully!");
            state.store(true);
        } else {
            Self::alert("Login failed!");
            state.store(false);
        }
    }
}
