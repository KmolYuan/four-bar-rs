use super::IoCtx;
use eframe::egui::{TextEdit, Ui};
use ehttp::{fetch, Request};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    fn get_link() -> String;
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Remote {
    address: String,
    account: String,
    password: String,
}

impl Default for Remote {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let address = get_link();
        #[cfg(not(target_arch = "wasm32"))]
        let address = "http://localhost:8080/".to_string();
        Self {
            address,
            account: "guest".to_string(),
            password: String::new(),
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
            ui.text_edit_singleline(&mut self.account);
        });
        ui.horizontal(|ui| {
            ui.label("Password");
            ui.add(TextEdit::singleline(&mut self.password).password(true));
        });
        if ui.button("login").clicked() {
            let mut headers = BTreeMap::new();
            headers.insert("content-type".to_string(), "application/json".to_string());
            let body = format!(
                "{{\"account\": \"{}\", \"password\": \"{}\"}}",
                self.account, self.password
            );
            let req = Request {
                method: "POST".to_string(),
                url: format!("{}/login", self.address.trim_end_matches('/')),
                body: body.into_bytes(),
                headers,
            };
            fetch(req, |r| match r {
                Ok(r) if r.ok => IoCtx::alert("Login successfully!"),
                _ => IoCtx::alert("Login failed!"),
            });
        }
    }
}
