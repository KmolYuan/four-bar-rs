use plotters::{
    element::{Drawable, PointCollection},
    style::ShapeStyle,
};
use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

pub(crate) struct Ball<Coord> {
    center: Coord,
    p: Coord,
    style: ShapeStyle,
}

impl<Coord> Ball<Coord> {
    pub(crate) fn new<S>(center: Coord, p: Coord, style: S) -> Self
    where
        S: Into<ShapeStyle>,
    {
        Self { center, p, style: style.into() }
    }

    pub(crate) fn series(self) -> std::iter::Once<Self> {
        std::iter::once(self)
    }
}

impl<'a, Coord> PointCollection<'a, Coord> for &'a Ball<Coord> {
    type Point = &'a Coord;
    type IntoIter = std::array::IntoIter<&'a Coord, 2>;

    fn point_iter(self) -> Self::IntoIter {
        [&self.center, &self.p].into_iter()
    }
}

impl<Coord, DB: DrawingBackend> Drawable<DB> for Ball<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut pos: I,
        backend: &mut DB,
        _ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let center @ (x1, y1) = pos.next().unwrap();
        let (x2, y2) = pos.next().unwrap();
        let radius = (x1 as f64 - x2 as f64).hypot(y1 as f64 - y2 as f64).round();
        backend.draw_circle(center, radius as u32, &self.style, self.style.filled)
    }
}
