use eframe::egui::{emath::Numeric, DragValue, Ui};
use std::f64::consts::TAU;

pub(crate) fn switch(
    ui: &mut Ui,
    val: &mut bool,
    icon1: &'static str,
    tip1: &'static str,
    icon2: &'static str,
    tip2: &'static str,
) {
    if *val {
        if ui.small_button(icon1).on_hover_text(tip1).clicked() {
            *val = false;
        }
    } else if ui.small_button(icon2).on_hover_text(tip2).clicked() {
        *val = true;
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
    DragValue::new(val)
        .prefix(label)
        .clamp_range(1e-4..=f64::MAX)
        .min_decimals(2)
        .speed(n)
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
