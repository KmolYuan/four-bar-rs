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
        let mut is_previous_solid = false;
        for curr in points {
            let curr_f = to_f(curr);
            let (dx, dy) = (curr_f.0 - start.0, curr_f.1 - start.1);
            let mut d = dx.hypot(dy).max(f32::EPSILON);
            let scale = size / d;
            let gap_scale = spacing / d;
            // Start drawing until last segment
            // 1) o-- --  o  (need to patch last one)
            // 2) o-- -- o   (ignore the last one)
            // 3) o o        (points are too dense)
            if is_previous_solid {
                start.0 += dx * gap_scale;
                start.1 += dy * gap_scale;
                d -= spacing;
            }
            while d >= size {
                // Solid line
                let end = (start.0 + dx * scale, start.1 + dy * scale);
                backend.draw_path([to_i(start), to_i(end)], &self.style)?;
                // Spacing
                start = (end.0 + dx * gap_scale, end.1 + dy * gap_scale);
                d -= size + spacing;
            }
            // Finish the last segment
            // 1) o-- -- -o  (patched)
            // 2) o-o        (become solid line)
            let line = [to_i(start), curr];
            is_previous_solid = d > 0. && line[0] != line[1];
            if is_previous_solid {
                backend.draw_path(line, &self.style)?;
            }
            // Move to the current point
            start = curr_f;
        }
        Ok(())
    }
}
