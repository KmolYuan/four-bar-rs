//! The functions used to plot the curve and synthesis result.

use crate::{curve, FourBar};
#[doc(no_inline)]
pub use plotters::{prelude::*, *};

type PResult<T, B> = Result<T, DrawingAreaErrorKind<<B as DrawingBackend>::ErrorType>>;

#[inline]
fn font() -> TextStyle<'static> {
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
    let cap = format!("Convergence Plot (Best Fitness: {:.04})", best_f);
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

/// Drawing option of four-bar linkage and its input angle.
#[derive(Default)]
pub struct Opt<'a> {
    fb: Option<FourBar>,
    angle: Option<f64>,
    title: Option<&'a str>,
    dot: bool,
    grid: bool,
}

impl From<Option<Self>> for Opt<'_> {
    fn from(opt: Option<Self>) -> Self {
        opt.unwrap_or_default()
    }
}

impl<F: Into<FourBar>> From<F> for Opt<'_> {
    fn from(fb: F) -> Self {
        Self { fb: Some(fb.into()), ..Self::default() }
    }
}

impl<'a> Opt<'a> {
    /// Create a default option, enables nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the linkage.
    pub fn fb(self, fb: impl Into<FourBar>) -> Self {
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

    /// Use grid in the plot.
    pub fn use_grid(self, grid: bool) -> Self {
        Self { grid, ..self }
    }

    fn joints(&self) -> Option<[[f64; 2]; 5]> {
        let fb = self.fb.as_ref()?;
        let [start, end] = fb.angle_bound().expect("invalid linkage");
        let angle = match self.angle {
            Some(angle) if (start..end).contains(&angle) => angle,
            _ => start + (end - start) * 0.25,
        };
        Some(fb.pos(angle))
    }
}

/// Plot 2D curves and linkages.
pub fn plot2d<'a, B, O>(backend: B, curves: &[(&str, &[[f64; 2]])], opt: O) -> PResult<(), B>
where
    B: DrawingBackend,
    O: Into<Opt<'a>>,
{
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;
    let opt = opt.into();
    let joints = opt.joints();
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
    let mut mesh = chart.configure_mesh();
    if !opt.grid {
        mesh.disable_mesh();
    }
    mesh.x_label_style(font()).y_label_style(font()).draw()?;
    for (i, &(label, curve)) in curves.iter().enumerate() {
        let curve = curve::get_valid_part(curve);
        let color = Palette99::pick(i);
        let anno = if opt.dot {
            if i % 2 == 1 {
                chart.draw_series(curve.iter().map(|&[x, y]| Circle::new((x, y), 5, &color)))?
            } else {
                let series = curve
                    .iter()
                    .map(|&[x, y]| TriangleMarker::new((x, y), 5, &color));
                chart.draw_series(series)?
            }
        } else {
            chart.draw_series(LineSeries::new(curve.iter().map(|&[x, y]| (x, y)), &color))?
        };
        anno.label(label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
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
