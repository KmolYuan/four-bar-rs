use eframe::egui::plot::{Value, Values};

#[inline]
pub fn as_values(iter: &[[f64; 2]]) -> Values {
    Values::from_values_iter(iter.iter().map(|&[x, y]| Value::new(x, y)))
}

#[inline]
pub fn as_values_lin(iter: &[f64]) -> Values {
    let iter = iter
        .iter()
        .enumerate()
        .map(|(x, y)| Value::new(x as f64, *y));
    Values::from_values_iter(iter)
}
