//! The functions used to plot the 2D curve and synthesis result.

use crate::*;
#[doc(no_inline)]
pub use plotters::{prelude::*, *};

pub(crate) type PResult<T, B> = Result<T, DrawingAreaErrorKind<<B as DrawingBackend>::ErrorType>>;

macro_rules! impl_opt {
    ($(
        $(#[$meta:meta])+
        struct $ty_name:ident { $inner:ty, $coord:ty }
    )+) => {$(
        $(#[$meta])+
        #[derive(Default)]
        pub struct $ty_name<'a> {
            fb: Option<$inner>,
            angle: Option<f64>,
            title: Option<&'a str>,
            dot: bool,
        }

        impl From<Option<Self>> for $ty_name<'_> {
            fn from(opt: Option<Self>) -> Self {
                opt.unwrap_or_default()
            }
        }

        impl<F: Into<$inner>> From<F> for $ty_name<'_> {
            fn from(fb: F) -> Self {
                Self { fb: Some(fb.into()), ..Self::default() }
            }
        }

        impl<'a> $ty_name<'a> {
            /// Create a default option, enables nothing.
            pub fn new() -> Self {
                Self::default()
            }

            /// Set the linkage.
            pub fn fb(self, fb: impl Into<$inner>) -> Self {
                Self { fb: Some(fb.into()), ..self }
            }

            /// Set the input angle of the linkage.
            ///
            /// If the angle value is not in the range of [`FourBar::angle_bound()`],
            /// the actual angle will be the midpoint.
            pub fn angle(self, angle: f64) -> Self {
                Self { angle: angle.into(), ..self }
            }

            /// Set the title.
            pub fn title(self, s: &'a str) -> Self {
                Self { title: Some(s), ..self }
            }

            /// Use dot in the curves.
            pub fn use_dot(self, dot: bool) -> Self {
                Self { dot, ..self }
            }

            fn joints(&self) -> Option<[$coord; 5]> {
                let fb = self.fb.as_ref()?;
                let [start, end] = fb.angle_bound().expect("invalid linkage");
                let angle = match self.angle {
                    Some(angle) if (start..end).contains(&angle) => angle,
                    _ => start + (end - start) * 0.25,
                };
                Some(fb.pos(angle))
            }
        }
    )+};
}

pub(crate) use impl_opt;

#[inline]
pub(crate) fn font() -> TextStyle<'static> {
    ("Times New Roman", 24).into_font().color(&BLACK)
}

/// Plot the synthesis history.
pub fn history<B, H>(backend: B, history: H) -> PResult<(), B>
where
    B: DrawingBackend,
    H: AsRef<[f64]>,
{
    let history = history.as_ref();
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;
    let best_f = history.last().unwrap();
    let cap = format!("Convergence Plot (Best Fitness: {best_f:.04})");
    let max_fitness = history
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(cap, font())
        .set_label_area_size(LabelAreaPosition::Left, (10).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (6).percent())
        .margin((8).percent())
        .build_cartesian_2d(0..history.len() - 1, 0.0..*max_fitness)?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_desc("Generation")
        .x_label_style(font())
        .y_desc("Fitness")
        .y_label_style(font())
        .draw()?;
    chart.draw_series(LineSeries::new(history.iter().copied().enumerate(), BLUE))?;
    Ok(())
}

impl_opt! {
    /// Drawing option of four-bar linkage and its input angle.
    struct Opt { FourBar, [f64; 2] }
}

/// Plot 2D curves and linkages.
pub fn plot<'a, B, C, O>(backend: B, curves: C, opt: O) -> PResult<(), B>
where
    B: DrawingBackend,
    C: IntoIterator<Item = (&'a str, &'a [[f64; 2]])>,
    O: Into<Opt<'a>>,
{
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;
    let opt = opt.into();
    let joints = opt.joints();
    let curves = curves.into_iter().collect::<Vec<_>>();
    let iter = curves.iter().flat_map(|(_, curve)| curve.iter());
    let [x_min, x_max, y_min, y_max] = bounding_box(iter.chain(joints.iter().flatten()));
    let mut chart = ChartBuilder::on(&root);
    if let Some(title) = opt.title {
        chart.caption(title, font());
    }
    let mut chart = chart
        .set_label_area_size(LabelAreaPosition::Left, (8).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
        .margin((8).percent())
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    chart
        .configure_mesh()
        .x_label_style(font())
        .y_label_style(font())
        .draw()?;
    for (i, &(label, curve)) in curves.iter().enumerate() {
        let curve = curve::get_valid_part(curve);
        let color = Palette99::pick(i);
        if opt.dot {
            if i % 2 == 1 {
                chart
                    .draw_series(curve.iter().map(|&[x, y]| Circle::new((x, y), 5, &color)))?
                    .label(label)
                    .legend(move |(x, y)| Circle::new((x + 10, y), 5, &color));
            } else {
                let series = curve
                    .iter()
                    .map(|&[x, y]| TriangleMarker::new((x, y), 5, &color));
                chart
                    .draw_series(series)?
                    .label(label)
                    .legend(move |(x, y)| TriangleMarker::new((x + 10, y), 5, &color));
            }
        } else {
            chart
                .draw_series(LineSeries::new(curve.iter().map(|&[x, y]| (x, y)), &color))?
                .label(label)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
        }
    }
    if let Some(joints @ [p0, p1, p2, p3, p4]) = joints {
        for line in [[p0, p2].as_slice(), &[p2, p4, p3, p2], &[p1, p3]] {
            chart.draw_series(LineSeries::new(line.iter().map(|&[x, y]| (x, y)), BLACK))?;
        }
        let grounded = joints[..2].iter().map(|&[x, y]| {
            EmptyElement::at((x, y)) + TriangleMarker::new((0, 10), 10, BLACK.filled())
        });
        chart.draw_series(grounded)?;
        let joints = joints
            .iter()
            .map(|&[x, y]| Circle::new((x, y), 5, BLACK.filled()));
        chart.draw_series(joints)?;
    }
    if curves.iter().filter(|(_, c)| !c.is_empty()).count() > 1 {
        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::LowerRight)
            .background_style(WHITE)
            .border_style(BLACK)
            .label_font(font())
            .draw()?;
    }
    Ok(())
}

/// Get the 1:1 bounding box of the data, ignore the labels.
pub fn bounding_box<'a>(pts: impl IntoIterator<Item = &'a [f64; 2]>) -> [f64; 4] {
    let [mut x_min, mut x_max] = [&f64::INFINITY, &-f64::INFINITY];
    let [mut y_min, mut y_max] = [&f64::INFINITY, &-f64::INFINITY];
    for [x, y] in pts {
        if x < x_min {
            x_min = x;
        }
        if x > x_max {
            x_max = x;
        }
        if y < y_min {
            y_min = y;
        }
        if y > y_max {
            y_max = y;
        }
    }
    let dx = (x_max - x_min).abs();
    let dy = (y_max - y_min).abs();
    if dx > dy {
        let cen = (y_min + y_max) * 0.5;
        let r = dx * 0.5;
        [*x_min, *x_max, cen - r, cen + r]
    } else {
        let cen = (x_min + x_max) * 0.5;
        let r = dy * 0.5;
        [cen - r, cen + r, *y_min, *y_max]
    }
}