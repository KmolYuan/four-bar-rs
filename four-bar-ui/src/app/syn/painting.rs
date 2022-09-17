use eframe::egui::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(super) struct Painting {
    stroke: Stroke,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
        }
    }
}

impl Painting {
    pub(super) fn ui(&mut self, ui: &mut Ui, line: &mut Vec<[f64; 2]>) -> Response {
        stroke_ui(ui, &mut self.stroke, "Stroke");
        Frame::canvas(ui.style())
            .show(ui, |ui| self.paint(ui, line))
            .inner
    }

    fn paint(&self, ui: &mut Ui, line: &mut Vec<[f64; 2]>) -> Response {
        let (mut res, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());
        let to_screen = {
            let points = line
                .iter()
                .map(|&[x, y]| Pos2::new(x as f32, -y as f32))
                .collect::<Vec<_>>();
            let from = Rect::from_points(&points);
            emath::RectTransform::from_to(from, res.rect)
        };
        let from_screen = to_screen.inverse();
        if let Some(pointer_pos) = res.interact_pointer_pos() {
            let Pos2 { x, y } = from_screen * pointer_pos;
            let c = [x as f64, -y as f64];
            if line.last() != Some(&c) {
                line.push(c);
                res.mark_changed();
            }
        } else if !line.is_empty() {
            res.mark_changed();
        }
        if line.len() >= 2 {
            let points = line
                .iter()
                .map(|&[x, y]| to_screen * Pos2::new(x as f32, -y as f32))
                .collect();
            painter.add(Shape::line(points, self.stroke));
        }
        res
    }
}
