use eframe::egui::plot::{Value, Values};

pub(crate) trait AsValues {
    fn as_values(&self) -> Values;
}

impl AsValues for [[f64; 2]] {
    fn as_values(&self) -> Values {
        let v = self
            .iter()
            .map(|&[x, y]| Value::new(x, y))
            .collect::<Vec<_>>();
        Values::from_values(v)
    }
}
