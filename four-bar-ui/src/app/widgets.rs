use eframe::egui::{emath::Numeric, DragValue, Ui};
use std::f64::consts::TAU;

pub(crate) fn switch_same(ui: &mut Ui, icon: &'static str, tip: &'static str, val: &mut bool) {
    if ui.small_button(icon).on_hover_text(tip).clicked() {
        *val = !*val;
    }
}

pub(crate) fn url_button(ui: &mut Ui, icon: &'static str, tip: &'static str, url: &'static str) {
    if ui.small_button(icon).on_hover_text(tip).clicked() {
        ui.ctx().output().open_url(url);
    }
}

pub(crate) fn unit<'a, V, N>(label: &'static str, val: &'a mut V, n: N) -> DragValue<'a>
where
    V: Numeric,
    N: Into<f64>,
{
    DragValue::new(val).prefix(label).speed(n)
}

pub(crate) fn link<'a>(label: &'static str, val: &'a mut f64, n: f64) -> DragValue<'a> {
    unit(label, val, n)
        .clamp_range(1e-4..=f64::MAX)
        .min_decimals(2)
}

pub(crate) fn angle(ui: &mut Ui, label: &'static str, val: &mut f64, suffix: &'static str) {
    ui.horizontal(|ui| {
        if suffix.is_empty() {
            *val = val.rem_euclid(TAU);
        }
        let mut deg = val.to_degrees();
        let dv = DragValue::new(&mut deg)
            .prefix(label)
            .suffix(" deg".to_string() + suffix)
            .min_decimals(2)
            .speed(1.);
        if ui.add(dv).changed() {
            *val = deg.to_radians();
        }
        let dv = DragValue::new(val)
            .suffix(" rad".to_string() + suffix)
            .min_decimals(2)
            .speed(0.01);
        ui.add(dv);
    });
}