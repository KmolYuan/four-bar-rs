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
//! fig.plot(root_l).unwrap();
//! fig.plot(root_r).unwrap();
//! ```
use crate::*;
#[doc(no_inline)]
pub use plotters::{prelude::*, *};
use std::{borrow::Cow, rc::Rc};

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
type LineData<'a, const N: usize> = (Cow<'a, str>, Cow<'a, [[f64; N]]>, Style, ShapeStyle);

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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Style {
    /// Continuous Line
    Line,
    /// Circle Marker
    #[default]
    Circle,
    /// Triangle Marker
    Triangle,
    /// Cross Marker
    Cross,
    /// Square Marker
    Square,
}

impl Style {
    /// Get the style names.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Line => "Line",
            Self::Circle => "Circle",
            Self::Triangle => "Triangle",
            Self::Cross => "Cross",
            Self::Square => "Square",
        }
    }

    pub(crate) fn draw<'a, DB, CT, I>(
        &self,
        chart: &mut ChartContext<'a, DB, CT>,
        line: I,
        color: ShapeStyle,
        label: &str,
    ) -> PResult<(), DB>
    where
        DB: DrawingBackend + 'a,
        CT: CoordTranslate,
        CT::From: Clone + 'static,
        I: IntoIterator<Item = CT::From>,
    {
        let color = color.stroke_width(color.stroke_width + 2);
        let dot_size = color.stroke_width + 5;
        let has_label = !label.is_empty();
        macro_rules! impl_marker {
            ($mk:ident) => {{
                let line = line.into_iter().map(|c| $mk::new(c, dot_size, color));
                let anno = chart.draw_series(line)?;
                if has_label {
                    anno.label(label).legend(move |(x, y)| {
                        $mk::new((x + dot_size as i32 / 2, y), dot_size, color)
                    });
                }
            }};
        }
        match self {
            Self::Line => {
                let line = LineSeries::new(line, color);
                let anno = chart.draw_series(line)?;
                if has_label {
                    anno.label(label)
                        .legend(move |(x, y)| PathElement::new([(x, y), (x + 20, y)], color));
                }
            }
            Self::Circle => impl_marker!(Circle),
            Self::Triangle => impl_marker!(TriangleMarker),
            Self::Cross => impl_marker!(Cross),
            Self::Square => {
                let r = dot_size as i32;
                let line = line
                    .into_iter()
                    .map(|c| EmptyElement::at(c) + Rectangle::new([(r, r), (-r, -r)], color));
                let anno = chart.draw_series(line)?;
                if has_label {
                    anno.label(label).legend(move |(x, y)| {
                        EmptyElement::at((x + dot_size as i32 / 2, y))
                            + Rectangle::new([(r, r), (-r, -r)], color)
                    });
                }
            }
        }
        Ok(())
    }
}

/// Legend position option.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum LegendPos {
    /// Hide Legend
    Hide,
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

impl LegendPos {
    /// Get the option names.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Hide => "Hide",
            Self::UL => "Upper Left",
            Self::ML => "Middle Left",
            Self::LL => "Lower Left",
            Self::UM => "Upper Middle",
            Self::MM => "Middle Middle",
            Self::LM => "Lower Middle",
            Self::UR => "Upper Right",
            Self::MR => "Middle Right",
            Self::LR => "Lower Right",
            Self::Coord(_, _) => "Coordinate",
        }
    }

    /// Transform to plotters option.
    pub fn to_plotter_pos(&self) -> Option<SeriesLabelPosition> {
        use SeriesLabelPosition::*;
        Some(match self {
            Self::Hide => None?,
            Self::UL => UpperLeft,
            Self::ML => MiddleLeft,
            Self::LL => LowerLeft,
            Self::UM => UpperMiddle,
            Self::MM => MiddleMiddle,
            Self::LM => LowerMiddle,
            Self::UR => UpperRight,
            Self::MR => MiddleRight,
            Self::LR => LowerRight,
            &Self::Coord(x, y) => Coordinate(x, y),
        })
    }
}

/// Option type base.
#[derive(Clone)]
pub struct FigureBase<'a, 'b, M, const N: usize> {
    pub(crate) fb: Option<&'b M>,
    angle: Option<f64>,
    lines: Vec<Rc<LineData<'a, N>>>,
    pub(crate) opt: Opt<'a>,
}

impl<M, const N: usize> Default for FigureBase<'_, '_, M, N> {
    fn default() -> Self {
        Self {
            fb: None,
            angle: None,
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

    /// Set the font family.
    pub fn font_family(mut self, family: impl Into<Cow<'a, str>>) -> Self {
        self.opt.font_family.replace(family.into());
        self
    }

    inner_opt! {
        /// Set the line stroke of the linkage.
        fn stroke(u32)
        /// Set font size.
        fn font(f64)
        /// Use grid in the plot.
        fn grid(bool)
        /// Show the axis.
        fn axis(bool)
        /// Set legend position.
        fn legend(LegendPos)
    }

    /// Add a line.
    pub fn add_line<S, L, C>(mut self, name: S, line: L, style: Style, color: C) -> Self
    where
        S: Into<Cow<'a, str>>,
        L: Into<Cow<'a, [[f64; N]]>>,
        C: Into<ShapeStyle>,
    {
        let line_data = (name.into(), line.into(), style, color.into());
        self.lines.push(Rc::new(line_data));
        self
    }

    // Iterate over lines
    pub(crate) fn lines(&self) -> impl Iterator<Item = &LineData<'a, N>> {
        self.lines.iter().map(|packed| &**packed)
    }

    /// Set the inner options.
    pub fn with_opt(self, opt: Opt<'a>) -> Self {
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

    // (stroke, dot_size)
    pub(crate) fn get_dot_size(&self) -> (u32, u32) {
        (self.stroke, self.stroke + 3)
    }

    #[inline]
    fn get_family(&self) -> &str {
        const DEFAULT_FONT: &str = "Times New Roman";
        self.opt
            .font_family
            .as_ref()
            .map(|s| s.as_ref())
            .unwrap_or(DEFAULT_FONT)
    }

    pub(crate) fn get_font(&self) -> TextStyle {
        (self.get_family(), self.opt.font).into_font().color(&BLACK)
    }

    pub(crate) fn get_axis_font(&self) -> TextStyle {
        (self.get_family(), self.opt.font * 0.8)
            .into_font()
            .color(&BLACK)
    }
}

impl<'a, M, const N: usize> std::ops::Deref for FigureBase<'a, '_, M, N> {
    type Target = Opt<'a>;
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
pub struct Opt<'a> {
    /// Stroke size
    pub stroke: u32,
    /// Font size
    pub font: f64,
    /// Font family
    pub font_family: Option<Cow<'a, str>>,
    /// Show grid
    pub grid: bool,
    /// Show axis
    pub axis: bool,
    /// Legend position
    pub legend: LegendPos,
}

impl Default for Opt<'_> {
    fn default() -> Self {
        Self {
            stroke: 5,
            font: 45.,
            font_family: None,
            grid: false,
            axis: true,
            legend: LegendPos::Hide,
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
        .x_label_style(font.clone())
        .y_desc("Fitness")
        .y_label_style(font.clone())
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
    ///     .add_line("First Curve", fb.curve(180), Style::Line, BLACK)
    ///     .plot(SVGBackend::with_string(&mut buf, (800, 800)))
    ///     .unwrap();
    /// ```
    pub fn plot<B, R>(&self, root: R) -> PResult<(), B>
    where
        B: DrawingBackend,
        Canvas<B>: From<R>,
    {
        let root = Canvas::from(root);
        root.fill(&WHITE)?;
        let (stroke, dot_size) = self.get_dot_size();
        let joints = self.get_joints();
        let Opt { grid, axis, legend, .. } = self.opt;
        let iter = self.lines().flat_map(|(_, curve, ..)| curve.iter());
        let [x_min, x_max, y_min, y_max] = bounding_box(iter.chain(joints.iter().flatten()));
        let mut chart = ChartBuilder::on(&root)
            .set_label_area_size(LabelAreaPosition::Left, (8).percent())
            .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
            .margin((4).percent())
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
        let mut mesh = chart.configure_mesh();
        // Draw mesh
        if !grid {
            mesh.disable_mesh();
        }
        if !axis {
            mesh.disable_axes();
        }
        mesh.label_style(self.get_axis_font()).draw()?;
        // Draw curve
        for (label, line, style, color) in self.lines() {
            let line = line.iter().map(|&[x, y]| (x, y));
            style.draw(&mut chart, line, *color, label)?;
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
        }
        if let Some(legend) = legend.to_plotter_pos() {
            chart
                .configure_series_labels()
                .position(legend)
                .background_style(WHITE)
                .border_style(BLACK)
                .label_font(self.get_font())
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
