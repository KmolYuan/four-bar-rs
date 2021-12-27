use super::{Atomic, IoCtx};
use eframe::egui::{TextEdit, Ui};
use serde::{Deserialize, Serialize};

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
    is_connected: bool,
    #[serde(skip)]
    is_login: Atomic<bool>,
}

impl Default for Remote {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let address = IoCtx::get_host();
        #[cfg(not(target_arch = "wasm32"))]
        let address = "http://localhost:8080/".to_string();
        Self {
            address,
            info: Default::default(),
            is_connected: false,
            is_login: Atomic::from(false),
        }
    }
}

impl Remote {
    pub(crate) fn is_login(&self) -> bool {
        self.is_login.load()
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui, ctx: &IoCtx) {
        ui.heading("Cloud Computing Service");
        if self.is_connected {
            if self.is_login.load() {
                self.after_login(ui, ctx);
            } else {
                self.before_login(ui, ctx);
            }
        } else {
            #[cfg(target_arch = "wasm32")]
            let _ = self.connect(ctx);
            #[cfg(not(target_arch = "wasm32"))]
            let _ = self.before_connect(ui, ctx);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn before_connect(&mut self, ui: &mut Ui, ctx: &IoCtx) {
        ui.horizontal(|ui| {
            ui.label("Address");
            ui.text_edit_singleline(&mut self.address);
        });
        if ui.button("Connect").clicked() {
            self.connect(ctx);
        }
    }

    fn connect(&mut self, ctx: &IoCtx) {
        let id = ctx.identity(&self.address);
        self.is_connected = id.is_some();
        match id {
            Some(id) => {
                if !id.is_empty() {
                    self.info.account = id;
                    self.is_login.store(true);
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            None => IoCtx::alert("Connection failed!"),
            #[cfg(target_arch = "wasm32")]
            None => unreachable!(),
        }
    }

    fn before_login(&mut self, ui: &mut Ui, ctx: &IoCtx) {
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

    fn after_login(&mut self, ui: &mut Ui, ctx: &IoCtx) {
        ui.horizontal(|ui| {
            ui.label("Account");
            ui.label(&self.info.account);
        });
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
    }
}
