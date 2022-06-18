use eframe::egui::*;
use std::f64::consts::TAU;

pub fn url_button(ui: &mut Ui, icon: &str, tip: &str, url: &str) {
    if ui.small_button(icon).on_hover_text(tip).clicked() {
        ui.ctx().output().open_url(url);
    }
}

pub fn unit<V, N>(ui: &mut Ui, label: &str, val: &mut V, n: N) -> Response
where
    V: emath::Numeric,
    N: Into<f64>,
{
    ui.add(DragValue::new(val).prefix(label).speed(n))
}

pub fn link(ui: &mut Ui, label: &str, val: &mut f64, n: f64) -> Response {
    let dv = DragValue::new(val)
        .prefix(label)
        .speed(n)
        .clamp_range(1e-4..=f64::MAX)
        .min_decimals(2);
    ui.add(dv)
}

pub fn angle(ui: &mut Ui, label: &str, val: &mut f64, suffix: &str) -> Response {
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
