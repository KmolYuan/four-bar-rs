use eframe::egui::plot::*;

pub(crate) fn as_values(iter: &[[f64; 2]]) -> Values {
    Values::from_values_iter(iter.into_iter().map(|&[x, y]| Value::new(x, y)))
}
