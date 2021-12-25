use super::{Atomic, IoCtx};
use eframe::egui::{TextEdit, Ui};
use hmac_sha512::Hash;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    fn get_link() -> String;
}

/// Sha512 encrypt function.
pub fn sha512(s: &str) -> String {
    Hash::hash(s).map(|n| format!("{:02x?}", n)).join("")
}

/// Store the login information.
#[derive(Deserialize, Serialize, Clone)]
pub struct LoginInfo {
    pub account: String,
    pub password: String,
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
            sha512(&self.account),
            sha512(&self.password)
        )
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Remote {
    address: String,
    info: LoginInfo,
    #[serde(skip)]
    is_login: Atomic<bool>,
}

impl Remote {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn with_address(a: impl ToString) -> Self {
        Self {
            address: a.to_string(),
            ..Self::default()
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui, ctx: &IoCtx) {
        ui.heading("Cloud Computing Service");
        ui.horizontal(|ui| {
            ui.label("Address");
            #[cfg(target_arch = "wasm32")]
            let _ = ui.label(get_link());
            #[cfg(not(target_arch = "wasm32"))]
            let _ = ui.text_edit_singleline(&mut self.address);
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
            ctx.login(&self.address, &self.info.account, &self.info.to_json());
        }
    }
}
