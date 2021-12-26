use super::{Atomic, IoCtx};
use eframe::egui::{TextEdit, Ui};
use hmac_sha512::Hash;
use serde::{Deserialize, Serialize};

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
        Self {
            address: "http://localhost:8080/".to_string(),
            info: Default::default(),
            is_login: Default::default(),
        }
    }
}

impl Remote {
    pub(crate) fn ui(&mut self, ui: &mut Ui, ctx: &IoCtx) {
        ui.heading("Cloud Computing Service");
        #[cfg(not(target_arch = "wasm32"))]
        let _ = ui.horizontal(|ui| {
            ui.label("Address");
            ui.text_edit_singleline(&mut self.address);
        });
        if self.is_login.load() {
            if ui.button("logout").clicked() {
                let is_login = self.is_login.clone();
                let done = move |b| {
                    if b {
                        IoCtx::alert("Logout successfully!");
                        is_login.store(false)
                    } else {
                        IoCtx::alert("Logout failed!");
                    }
                };
                ctx.logout(&self.address, done);
            }
        } else {
            ui.horizontal(|ui| {
                ui.label("Account");
                ui.text_edit_singleline(&mut self.info.account);
            });
            ui.horizontal(|ui| {
                ui.label("Password");
                ui.add(TextEdit::singleline(&mut self.info.password).password(true));
            });
            if ui.button("login").clicked() {
                let body = self.info.to_json();
                let is_login = self.is_login.clone();
                let done = move |b| {
                    if b {
                        IoCtx::alert("Login successfully!");
                    } else {
                        IoCtx::alert("Login failed!");
                    }
                    is_login.store(b)
                };
                ctx.login(&self.address, &self.info.account, &body, done);
            }
        }
    }
}
