use super::{Atomic, IoCtx};
use eframe::egui::{TextEdit, Ui};
use ehttp::{fetch, Request};
use hmac_sha512::Hash;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    fn get_link() -> String;
}

pub(crate) fn sha512(s: &str) -> String {
    Hash::hash(s).map(|n| format!("{:02x?}", n)).join("")
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct LoginInfo {
    pub(crate) account: String,
    pub(crate) password: String,
}

impl Default for LoginInfo {
    fn default() -> Self {
        Self {
            account: "guest".to_string(),
            password: String::new(),
        }
    }
}

impl LoginInfo {
    pub(crate) fn to_json(&self) -> String {
        format!(
            "{{\"account\": \"{}\", \"password\": \"{}\"}}",
            self.account,
            sha512(&self.password)
        )
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Remote {
    address: String,
    info: LoginInfo,
    #[serde(skip)]
    is_login: Atomic<bool>,
}

impl Default for Remote {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let address = get_link();
        #[cfg(not(target_arch = "wasm32"))]
        let address = "http://localhost:8080/".to_string();
        Self {
            address,
            is_login: Atomic::new(false),
            info: Default::default(),
        }
    }
}

impl Remote {
    pub(crate) fn ui(&mut self, ui: &mut Ui, _ctx: &IoCtx) {
        ui.heading("Cloud Computing Service");
        ui.horizontal(|ui| {
            ui.label("Address");
            ui.text_edit_singleline(&mut self.address);
        });
        ui.horizontal(|ui| {
            ui.label("Account");
            ui.text_edit_singleline(&mut self.info.account);
        });
        ui.horizontal(|ui| {
            ui.label("Password");
            ui.add(TextEdit::singleline(&mut self.info.password).password(true));
        });
        if ui.button("login").clicked() {
            let req = Request {
                method: "POST".to_string(),
                url: format!("{}/login", self.address.trim_end_matches('/')),
                body: self.info.to_json().into_bytes(),
                headers: Request::create_headers_map(&[("content-type", "application/json")]),
            };
            fetch(req, |r| match r {
                Ok(r) if r.ok => IoCtx::alert("Login successfully!"),
                _ => IoCtx::alert("Login failed!"),
            });
        }
    }
}
