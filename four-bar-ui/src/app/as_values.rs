use eframe::egui::plot::{Value, Values};

#[inline]
pub fn as_values(iter: &[[f64; 2]]) -> Values {
    Values::from_values_iter(iter.iter().map(|&[x, y]| Value::new(x, y)))
}
