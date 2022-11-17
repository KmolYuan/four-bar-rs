use eframe::egui::*;
use std::f64::consts::TAU;

pub(crate) fn url_btn(ui: &mut Ui, icon: &str, tip: &str, url: &str) {
    if ui.small_button(icon).on_hover_text(tip).clicked() {
        ui.ctx().output().open_url(url);
    }
}

pub(crate) fn unit<V, N>(ui: &mut Ui, label: &str, val: &mut V, int: N) -> Response
where
    V: emath::Numeric,
    N: Into<f64>,
{
    ui.add(DragValue::new(val).prefix(label).speed(int))
}

pub(crate) fn link(ui: &mut Ui, label: &str, val: &mut f64, int: f64) -> Response {
    let dv = DragValue::new(val)
        .prefix(label)
        .speed(int)
        .clamp_range(1e-4..=f64::MAX)
        .min_decimals(2);
    ui.add(dv)
}

pub(crate) fn angle(ui: &mut Ui, label: &str, val: &mut f64, suffix: &str) -> Response {
    ui.horizontal(|ui| {
        if suffix.is_empty() {
            *val = val.rem_euclid(TAU);
        }
        let mut deg = val.to_degrees();
        let dv = DragValue::new(&mut deg)
            .prefix(label)
            .suffix(format!(" deg{suffix}"))
            .min_decimals(2)
            .speed(1.);
        let res = ui.add(dv);
        if res.changed() {
            *val = deg.to_radians();
        }
        let dv = DragValue::new(val)
            .suffix(format!(" rad{suffix}"))
            .min_decimals(2)
            .speed(0.01);
        res | ui.add(dv)
    })
    .inner
}

pub(crate) fn percent(ui: &mut Ui, label: &str, val: &mut f64) -> Response {
    let dv = DragValue::new(val)
        .prefix(label)
        .custom_formatter(|v, _| format!("{}%", v * 100.))
        .speed(0.1)
        .clamp_range(1e-4..=f64::MAX)
        .min_decimals(2);
    ui.add(dv)
}
