use super::{to_f, to_i};
use plotters::{
    element::{Drawable, IntoDynElement, PointCollection},
    style::SizeDesc,
};
use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

pub(crate) struct DottedPath<I: Iterator + Clone, Size: SizeDesc, Marker> {
    points: I,
    shift: Size,
    spacing: Size,
    func: Box<dyn Fn(BackendCoord) -> Marker>,
}

impl<I: Iterator + Clone, Size: SizeDesc, Marker> DottedPath<I, Size, Marker> {
    pub(crate) fn new<I0, F>(points: I0, shift: Size, spacing: Size, func: F) -> Self
    where
        I0: IntoIterator<IntoIter = I>,
        F: Fn(BackendCoord) -> Marker + 'static,
    {
        Self {
            points: points.into_iter(),
            shift,
            spacing,
            func: Box::new(func),
        }
    }

    pub(crate) fn series(self) -> std::iter::Once<Self> {
        std::iter::once(self)
    }
}

impl<'a, I: Iterator + Clone, Size: SizeDesc, Marker> PointCollection<'a, I::Item>
    for &'a DottedPath<I, Size, Marker>
{
    type Point = I::Item;
    type IntoIter = I;

    fn point_iter(self) -> Self::IntoIter {
        self.points.clone()
    }
}

impl<I0, Size, DB, Marker> Drawable<DB> for DottedPath<I0, Size, Marker>
where
    I0: Iterator + Clone,
    Size: SizeDesc,
    DB: DrawingBackend,
    Marker: IntoDynElement<'static, DB, BackendCoord>,
{
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let mut shift = self.shift.in_pixels(&ps).max(0) as f32;
        let mut start = match points.next() {
            Some(start_i) => {
                // Draw the first marker if no shift
                if shift == 0. {
                    let mk = (self.func)(start_i).into_dyn();
                    mk.draw(mk.point_iter().iter().copied(), backend, ps)?;
                }
                to_f(start_i)
            }
            None => return Ok(()),
        };
        let spacing = self.spacing.in_pixels(&ps).max(0) as f32;
        let mut dist = 0.;
        for curr in points {
            let end = to_f(curr);
            // Loop for spacing
            while start != end {
                let (dx, dy) = (end.0 - start.0, end.1 - start.1);
                let d = dx.hypot(dy);
                let spacing = if shift == 0. { spacing } else { shift };
                let left = spacing - dist;
                // Set next point to `start`
                if left < d {
                    let t = left / d;
                    start = (start.0 + dx * t, start.1 + dy * t);
                    dist += left;
                } else {
                    start = end;
                    dist += d;
                }
                // Draw if needed
                if spacing <= dist {
                    let start_i = to_i(start);
                    let mk = (self.func)(start_i).into_dyn();
                    mk.draw(mk.point_iter().iter().copied(), backend, ps)?;
                    shift = 0.;
                    dist = 0.;
                }
            }
        }
        Ok(())
    }
}
