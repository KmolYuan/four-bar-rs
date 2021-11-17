#[cfg(target_arch = "wasm32")]
use {
    js_sys::{Array, Function, JsString},
    wasm_bindgen::JsValue,
};
#[cfg(not(target_arch = "wasm32"))]
use {
    rfd::FileDialog,
    std::fs::{read_to_string, write},
};

#[cfg(target_arch = "wasm32")]
pub(crate) struct IoCtx {
    save_fn: Function,
    load_fn: Function,
    load_str: Array,
}

#[cfg(target_arch = "wasm32")]
impl Default for IoCtx {
    fn default() -> Self {
        IoCtx {
            save_fn: Function::new_no_args(""),
            load_fn: Function::new_no_args(""),
            load_str: Array::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub(crate) struct IoCtx;

impl IoCtx {
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn new(save_fn: Function, load_fn: Function) -> Self {
        Self {
            save_fn,
            load_fn,
            ..Self::default()
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn open(&self, ext: &[&str]) {
        let ext = ext
            .iter()
            .map(|s| format!(".{}", s))
            .collect::<Vec<_>>()
            .join(",");
        let this = JsValue::NULL;
        let format = JsValue::from(ext);
        self.load_fn.call2(&this, &self.load_str, &format).unwrap();
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
        let this = JsValue::NULL;
        let s = JsValue::from(s);
        let path = JsValue::from(file_name);
        self.save_fn.call2(&this, &s, &path).unwrap();
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
}
