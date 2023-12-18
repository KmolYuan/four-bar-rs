use plotters::{
    element::{Drawable, PointCollection},
    style::{ShapeStyle, SizeDesc},
};
use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

pub(crate) struct DashedPath<I: Iterator + Clone, Size: SizeDesc> {
    points: I,
    size: Size,
    spacing: Size,
    style: ShapeStyle,
}

impl<I: Iterator + Clone, Size: SizeDesc> DashedPath<I, Size> {
    pub(crate) fn new<I0, S>(points: I0, size: Size, spacing: Size, style: S) -> Self
    where
        I0: IntoIterator<IntoIter = I>,
        S: Into<ShapeStyle>,
    {
        Self {
            points: points.into_iter(),
            size,
            spacing,
            style: style.into(),
        }
    }

    pub(crate) fn series(self) -> std::iter::Once<Self> {
        std::iter::once(self)
    }
}

impl<'a, I: Iterator + Clone, Size: SizeDesc> PointCollection<'a, I::Item>
    for &'a DashedPath<I, Size>
{
    type Point = I::Item;
    type IntoIter = I;

    fn point_iter(self) -> Self::IntoIter {
        self.points.clone()
    }
}

impl<I0: Iterator + Clone, Size: SizeDesc, DB: DrawingBackend> Drawable<DB>
    for DashedPath<I0, Size>
{
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let to_i = |(x, y): (f32, f32)| (x.round() as i32, y.round() as i32);
        let to_f = |(x, y): (i32, i32)| (x as f32, y as f32);
        let mut start = match points.next() {
            Some(c) => to_f(c),
            None => return Ok(()),
        };
        let size = self.size.in_pixels(&ps).max(0) as f32;
        let spacing = self.spacing.in_pixels(&ps).max(0) as f32;
        let mut dist = 0.;
        let mut is_solid = true;
        let mut queue = vec![to_i(start)];
        for curr in points {
            let curr_f = to_f(curr);
            let (dx, dy) = (curr_f.0 - start.0, curr_f.1 - start.1);
            let d = dx.hypot(dy).max(f32::EPSILON);
            dist += d;
            if is_solid {
                if dist < size {
                    queue.push(curr);
                    start = curr_f;
                } else {
                    let t = (dist - size) / d;
                    start = (start.0 + dx * t, start.1 + dy * t);
                    queue.push(to_i(start));
                    backend.draw_path(queue.drain(..), &self.style)?;
                    dist = 0.;
                    is_solid = false;
                }
            } else if dist < spacing {
                start = curr_f;
            } else {
                let t = (dist - spacing) / d;
                start = (start.0 + dx * t, start.1 + dy * t);
                queue.push(to_i(start));
                dist = 0.;
                is_solid = true;
            }
        }
        if queue.len() > 1 {
            backend.draw_path(queue, &self.style)?;
        }
        Ok(())
    }
}
