#[cfg(target_arch = "wasm32")]
use {
    js_sys::{Array, JsString},
    wasm_bindgen::prelude::wasm_bindgen,
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
    #[wasm_bindgen(js_name = "saveFile")]
    fn save_file(s: &str, file_name: &str);
    #[wasm_bindgen(js_name = "loadFile")]
    fn load_file(arr: Array, format: &str);
}

#[cfg(target_arch = "wasm32")]
pub(crate) struct IoCtx {
    load_str: Array,
}

#[cfg(target_arch = "wasm32")]
impl Default for IoCtx {
    fn default() -> Self {
        IoCtx {
            load_str: Array::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub(crate) struct IoCtx;

impl IoCtx {
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn open(&self, ext: &[&str]) {
        let format = ext
            .iter()
            .map(|s| format!(".{}", s))
            .collect::<Vec<_>>()
            .join(",");
        load_file(self.load_str.clone(), &format);
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn open_result(&self) -> Option<String> {
        if self.load_str.length() > 0 {
            Some(String::from(JsString::from(self.load_str.pop())))
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
    #[allow(dead_code)]
    pub(crate) fn alert(&self, s: &str) {
        alert(s);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    pub(crate) fn alert(&self, s: &str) {
        MessageDialog::new().set_title("Alert").set_description(s);
    }
}
