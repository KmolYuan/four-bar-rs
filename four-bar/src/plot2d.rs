//! The functions used to plot the 2D curve and synthesis result.
//!
//! # Single Plot Example
//!
//! ```
//! use four_bar::plot2d::*;
//!
//! let fig = Figure::new().add_line("", vec![[0.; 2], [1.; 2]], Style::Line, BLACK);
//! let mut buf = String::new();
//! let svg = SVGBackend::with_string(&mut buf, (800, 800));
//! fig.plot(svg).unwrap();
//! ```
//!
//! # Sub-plots Example
//!
//! ```
//! use four_bar::plot2d::*;
//!
//! let fig = Figure::new().add_line("", vec![[0.; 2], [1.; 2]], Style::Line, BLACK);
//! let mut buf = String::new();
//! let svg = SVGBackend::with_string(&mut buf, (800, 800));
//! let (root_l, root_r) = svg.into_drawing_area().split_horizontally(800);
//! fig.clone().plot(root_l).unwrap();
//! fig.plot(root_r).unwrap();
//! ```
use crate::*;
#[doc(no_inline)]
pub use plotters::{prelude::*, *};
use std::borrow::Cow;

/// Drawing option of four-bar linkage and its input angle.
///
/// ```
/// use four_bar::{plot2d::Figure, FourBar};
/// // From linkage
/// let figure = Figure::from(&FourBar::example());
/// // Without linkage
/// let figure = Figure::new();
/// ```
pub type Figure<'a, 'b> = FigureBase<'a, 'b, FourBar, 2>;
pub(crate) type PResult<T, B> = Result<T, DrawingAreaErrorKind<<B as DrawingBackend>::ErrorType>>;
pub(crate) type Canvas<B> = DrawingArea<B, coord::Shift>;
pub(crate) type LineData<'a, const N: usize> = (Cow<'a, str>, Cow<'a, [[f64; N]]>, Style, RGBColor);

macro_rules! inner_opt {
    ($($(#[$meta:meta])+ fn $name:ident($ty:ty))+) => {$(
        $(#[$meta])+
        pub fn $name(mut self, $name: $ty) -> Self {
            self.opt.$name = $name;
            self
        }
    )+};
}

/// Line style.
#[derive(Clone, Copy)]
pub enum Style {
    /// Continuous Line
    Line,
    /// Circle Marker
    Circle,
    /// Triangle Marker
    Triangle,
    /// Cross Marker
    Cross,
    /// Square Marker
    Square,
}

/// Legend position option.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, PartialEq, Default)]
pub enum LegendPos {
    /// Upper Left
    UL,
    /// Middle Left
    ML,
    /// Lower Left
    LL,
    /// Upper Middle
    UM,
    /// Middle Middle
    MM,
    /// Lower Middle
    LM,
    /// Upper Right
    UR,
    /// Middle Right
    MR,
    /// Lower Right
    #[default]
    LR,
    /// Coordinate
    #[cfg_attr(feature = "clap", clap(skip))]
    Coord(i32, i32),
}

impl From<LegendPos> for SeriesLabelPosition {
    fn from(pos: LegendPos) -> Self {
        use LegendPos::*;
        match pos {
            UL => Self::UpperLeft,
            ML => Self::MiddleLeft,
            LL => Self::LowerLeft,
            UM => Self::UpperMiddle,
            MM => Self::MiddleMiddle,
            LM => Self::LowerMiddle,
            UR => Self::UpperRight,
            MR => Self::MiddleRight,
            LR => Self::LowerRight,
            Coord(x, y) => Self::Coordinate(x, y),
        }
    }
}

/// Option type base.
#[derive(Clone)]
pub struct FigureBase<'a, 'b, M, const N: usize> {
    pub(crate) fb: Option<&'b M>,
    pub(crate) angle: Option<f64>,
    pub(crate) title: Option<&'a str>,
    pub(crate) opt: Opt,
    pub(crate) lines: Vec<LineData<'a, N>>,
}

impl<M, const N: usize> Default for FigureBase<'_, '_, M, N> {
    fn default() -> Self {
        Self {
            fb: None,
            angle: None,
            title: None,
            opt: Default::default(),
            lines: Default::default(),
        }
    }
}

impl<'a, M, const N: usize> From<Option<&'a M>> for FigureBase<'_, 'a, M, N> {
    fn from(opt: Option<&'a M>) -> Self {
        match opt {
            Some(fb) => Self::from(fb),
            None => Self::new(),
        }
    }
}

impl<'a, M, const N: usize> From<&'a M> for FigureBase<'_, 'a, M, N> {
    fn from(fb: &'a M) -> Self {
        Self { fb: Some(fb), ..Self::default() }
    }
}

impl<'a, 'b, M, const N: usize> FigureBase<'a, 'b, M, N> {
    /// Create a default option.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach linkage.
    pub fn with_fb<'c>(self, fb: &'c M) -> FigureBase<'a, 'c, M, N> {
        FigureBase { fb: Some(fb), ..self }
    }

    /// Remove linkage.
    pub fn remove_fb(self) -> Self {
        Self { fb: None, ..self }
    }

    /// Set the input angle of the linkage.
    ///
    /// If the angle value is not in the range of [`FourBar::angle_bound()`],
    /// the actual angle will be the midpoint.
    pub fn angle(self, angle: f64) -> Self {
        Self { angle: Some(angle), ..self }
    }

    /// Set the title.
    pub fn title(self, title: &'a str) -> Self {
        Self { title: Some(title), ..self }
    }

    inner_opt! {
        /// Set the line stroke/point size.
        fn stroke(u32)
        /// Set font size.
        fn font(f64)
        /// Show the scale bar when the linkage is specified.
        fn scale_bar(bool)
        /// Use grid in the plot.
        fn grid(bool)
        /// Show the axis.
        fn axis(bool)
        /// Set legend position.
        fn legend(Option<LegendPos>)
    }

    /// Add a line.
    pub fn add_line<S, L>(mut self, name: S, line: L, style: Style, color: RGBColor) -> Self
    where
        S: Into<Cow<'a, str>>,
        L: Into<Cow<'a, [[f64; N]]>>,
    {
        self.lines.push((name.into(), line.into(), style, color));
        self
    }

    /// Set the inner options.
    pub fn with_opt(self, opt: Opt) -> Self {
        Self { opt, ..self }
    }

    pub(crate) fn get_joints<D: efd::EfdDim>(&self) -> Option<[efd::Coord<D>; 5]>
    where
        M: CurveGen<D>,
    {
        let fb = self.fb?;
        let [start, end] = fb.angle_bound().to_value()?;
        let angle = match self.angle {
            Some(angle) if (start..end).contains(&angle) => angle,
            _ => start + (end - start) * 0.25,
        };
        fb.pos(angle)
    }

    pub(crate) fn get_dot_size(&self) -> u32 {
        self.stroke + 3
    }
}

impl<M, const N: usize> std::ops::Deref for FigureBase<'_, '_, M, N> {
    type Target = Opt;
    fn deref(&self) -> &Self::Target {
        &self.opt
    }
}

/// 2D/3D plot option.
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(default)
)]
#[derive(Clone, PartialEq)]
pub struct Opt {
    /// Stroke size
    pub stroke: u32,
    /// Font size
    pub font: f64,
    /// Show scale bar
    pub scale_bar: bool,
    /// Show grid
    pub grid: bool,
    /// Show axis
    pub axis: bool,
    /// Legend position
    pub legend: Option<LegendPos>,
}

impl Default for Opt {
    fn default() -> Self {
        Self {
            stroke: 5,
            font: 24.,
            scale_bar: false,
            grid: false,
            axis: true,
            legend: None,
        }
    }
}

/// Plot the synthesis history.
pub fn history<B, R, H>(root: R, history: H) -> PResult<(), B>
where
    B: DrawingBackend,
    Canvas<B>: From<R>,
    H: AsRef<[f64]>,
{
    let font = ("Times New Roman", 24).into_font().color(&BLACK);
    let font = || font.clone();
    let history = history.as_ref();
    let root = Canvas::from(root);
    root.fill(&WHITE)?;
    let max_fitness = history
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let mut chart = ChartBuilder::on(&root)
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

impl Figure<'_, '_> {
    /// Plot 2D curves and linkages.
    ///
    /// ```
    /// use four_bar::{plot2d::*, FourBar};
    /// let fb = FourBar::example();
    /// let mut buf = String::new();
    /// Figure::from(&fb)
    ///     .axis(false)
    ///     .scale_bar(true)
    ///     .add_line("First Curve", fb.curve(180), Style::Line, BLACK)
    ///     .plot(SVGBackend::with_string(&mut buf, (800, 800)))
    ///     .unwrap();
    /// ```
    pub fn plot<B, R>(self, root: R) -> PResult<(), B>
    where
        B: DrawingBackend,
        Canvas<B>: From<R>,
    {
        let root = Canvas::from(root);
        root.fill(&WHITE)?;
        let dot_size = self.get_dot_size();
        let joints = self.get_joints();
        let Self {
            title,
            opt: Opt { stroke, font, scale_bar, grid, axis, legend },
            lines,
            ..
        } = self;
        let iter = lines.iter().flat_map(|(_, curve, ..)| curve.iter());
        let [x_min, x_max, y_min, y_max] = bounding_box(iter.chain(joints.iter().flatten()));
        let font = ("Times New Roman", font).into_font().color(&BLACK);
        let font = || font.clone();
        let mut chart = ChartBuilder::on(&root);
        if let Some(title) = title {
            chart.caption(title, font());
        }
        let mut chart = chart
            .set_label_area_size(LabelAreaPosition::Left, (8).percent())
            .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
            .margin((2).percent())
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
        let mut mesh = chart.configure_mesh();
        // Draw mesh
        if !grid {
            mesh.disable_mesh();
        }
        if !axis {
            mesh.disable_axes();
        }
        mesh.x_label_style(font()).y_label_style(font()).draw()?;
        // Draw curve
        for (label, line, style, color) in lines {
            macro_rules! marker {
                ($mk:ident) => {{
                    let line = line.iter().map(|&[x, y]| $mk::new((x, y), dot_size, color));
                    let anno = chart.draw_series(line)?;
                    if !label.is_empty() {
                        anno.label(label)
                            .legend(move |(x, y)| $mk::new((x, y), dot_size, color));
                    }
                }};
            }
            match style {
                Style::Line => {
                    let line = line.iter().map(|&[x, y]| (x, y));
                    let anno =
                        chart.draw_series(LineSeries::new(line, color.stroke_width(stroke)))?;
                    if !label.is_empty() {
                        anno.label(label).legend(move |(x, y)| {
                            PathElement::new([(x, y), (x + 20, y)], color.stroke_width(stroke))
                        });
                    }
                }
                Style::Triangle => marker!(TriangleMarker),
                Style::Cross => marker!(Cross),
                Style::Circle => marker!(Circle),
                Style::Square => {
                    let r = dot_size as i32 / 2;
                    let line = line.iter().map(|&[x, y]| {
                        EmptyElement::at((x, y)) + Rectangle::new([(r, r), (-r, -r)], color)
                    });
                    let anno = chart.draw_series(line)?;
                    if !label.is_empty() {
                        anno.label(label).legend(move |(x, y)| {
                            Rectangle::new([(x + r, y + r), (x - r, y - r)], color)
                        });
                    }
                }
            };
        }
        // Draw Linkage
        if let Some(joints @ [p0, p1, p2, p3, p4]) = joints {
            for line in [[p0, p2].as_slice(), &[p2, p4, p3, p2], &[p1, p3]] {
                let line = line.iter().map(|&[x, y]| (x, y));
                chart.draw_series(LineSeries::new(line, BLACK.stroke_width(stroke)))?;
            }
            let grounded = joints[..2].iter().map(|&[x, y]| {
                EmptyElement::at((x, y))
                    + TriangleMarker::new((0, 10), dot_size + 3, BLACK.filled())
            });
            chart.draw_series(grounded)?;
            let joints = joints
                .iter()
                .map(|&[x, y]| Circle::new((x, y), dot_size, BLACK.filled()));
            chart.draw_series(joints)?;
            // Draw scale bar
            if scale_bar {
                let scale_bar = scale_bar_size((x_max - x_min).min(y_max - y_min));
                for (p, color) in [
                    ((p0[0] + scale_bar, p0[1]), RED),
                    ((p0[0], p0[1] + scale_bar), BLUE),
                ] {
                    let style = color.stroke_width(stroke);
                    chart.draw_series(LineSeries::new([(p0[0], p0[1]), p], style))?;
                }
            }
        }
        if let Some(legend) = legend {
            chart
                .configure_series_labels()
                .position(legend.into())
                .background_style(WHITE)
                .border_style(BLACK)
                .label_font(font())
                .draw()?;
        }
        Ok(())
    }
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

/// Calculate the scale bar size.
pub fn scale_bar_size(x: f64) -> f64 {
    10f64.powi(x.log10().floor() as i32 - 1)
}
