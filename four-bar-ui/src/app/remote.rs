use super::Ctx;
use eframe::egui::*;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering::Relaxed},
    Arc,
};

/// Sha512 encrypt function.
pub fn sha512(s: &str) -> String {
    hmac_sha512::Hash::hash(s)
        .map(|n| format!("{:02x?}", n))
        .join("")
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
    pub fn to_json(&self) -> String {
        format!(
            "{{\"account\": \"{}\", \"password\": \"{}\"}}",
            self.account,
            sha512(&self.password)
        )
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Remote {
    address: String,
    info: LoginInfo,
    #[serde(skip)]
    is_connected: bool,
    #[serde(skip)]
    is_login: Arc<AtomicBool>,
}

impl Default for Remote {
    fn default() -> Self {
        Self {
            address: Ctx::get_host(),
            info: LoginInfo::default(),
            is_connected: false,
            is_login: Default::default(),
        }
    }
}

impl Remote {
    pub fn show(&mut self, ui: &mut Ui, ctx: &Ctx) {
        ui.heading("Cloud Computing Service");
        if self.is_connected {
            if self.is_login.load(Relaxed) {
                self.after_login(ui, ctx);
            } else {
                self.before_login(ui, ctx);
            }
        } else {
            self.before_connect(ui, ctx);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn before_connect(&mut self, _ui: &mut Ui, ctx: &Ctx) {
        self.connect(ctx);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn before_connect(&mut self, ui: &mut Ui, ctx: &Ctx) {
        ui.horizontal(|ui| {
            ui.label("Address");
            ui.text_edit_singleline(&mut self.address);
        });
        if ui.button("🖧 Connect").clicked() {
            self.connect(ctx);
        }
    }

    fn connect(&mut self, ctx: &Ctx) {
        match ctx.get_username(&self.address) {
            Some(id) => {
                self.is_connected = true;
                if !id.is_empty() {
                    self.info.account = id;
                    self.is_login.store(true, Relaxed);
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            None => Ctx::alert("Connection failed!"),
            #[cfg(target_arch = "wasm32")]
            None => unreachable!(),
        }
    }

    fn before_login(&mut self, ui: &mut Ui, ctx: &Ctx) {
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
                    Ctx::alert("Login successfully!");
                } else {
                    Ctx::alert("Login failed!");
                }
                is_login.store(b, Relaxed)
            };
            ctx.login(&self.address, &self.info.account, &body, done);
        }
    }

    fn after_login(&mut self, ui: &mut Ui, ctx: &Ctx) {
        ui.horizontal(|ui| {
            ui.label("Account");
            ui.label(&self.info.account);
        });
        if ui.button("logout").clicked() {
            let is_login = self.is_login.clone();
            let done = move |b| {
                if b {
                    Ctx::alert("Logout successfully!");
                    is_login.store(false, Relaxed);
                } else {
                    Ctx::alert("Logout failed!");
                }
            };
            ctx.logout(&self.address, done);
        }
    }
}
