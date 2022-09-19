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
            let rect = &res.rect;
            let use_w = rect.width() < rect.height();
            let min_rect = Rect::from_center_size(Pos2::ZERO, Vec2::new(100., 100.));
            let from = {
                let points = line
                    .iter()
                    .map(|&[x, y]| Pos2::new(x as f32, -y as f32))
                    .collect::<Vec<_>>();
                let rect = Rect::from_points(&points);
                if rect.is_finite() && !min_rect.contains_rect(rect) {
                    let w = if use_w { rect.width() } else { rect.height() };
                    Rect::from_center_size(rect.center(), Vec2::new(w, w))
                } else {
                    min_rect
                }
            };
            let w = if use_w { rect.width() } else { rect.height() };
            let to = Rect::from_center_size(rect.center(), Vec2::new(w, w));
            emath::RectTransform::from_to(from, to)
        };
        if let Some(pointer_pos) = res.interact_pointer_pos() {
            let from_screen = to_screen.inverse();
            let Pos2 { x, y } = from_screen * pointer_pos;
            let c = [x as f64, -y as f64];
            if line.last() != Some(&c) {
                line.push(c);
                res.mark_changed();
            }
        }
        if line.len() > 1 {
            let points = line
                .iter()
                .map(|&[x, y]| to_screen * Pos2::new(x as f32, -y as f32))
                .collect();
            painter.add(Shape::line(points, self.stroke));
        }
        res
    }
}
