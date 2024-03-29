//! Plot utitlities.
//!
//! Please see [`crate::plot::fb::Figure`] and [`crate::plot::sfb::Figure`] for
//! more information.
//!
//! # Single Plot Example
//!
//! ```
//! use four_bar::plot::*;
//!
//! let fig = fb::Figure::new(None).add_line("", vec![[0.; 2], [1.; 2]], Style::Line, BLACK);
//! let mut buf = String::new();
//! let svg = SVGBackend::with_string(&mut buf, (1600, 1600));
//! fig.plot(svg).unwrap();
//! ```
//!
//! # Sub-plots Example
//!
//! ```
//! use four_bar::plot::*;
//!
//! let fig = fb::Figure::new(None).add_line("", vec![[0.; 2], [1.; 2]], Style::Line, BLACK);
//! let mut buf = String::new();
//! let svg = SVGBackend::with_string(&mut buf, (1600, 800));
//! let (root_l, root_r) = svg.into_drawing_area().split_horizontally(800);
//! fig.plot(root_l).unwrap();
//! fig.plot(root_r).unwrap();
//! ```
use self::{ball::*, dashed_line::*, dotted_line::*};
use crate::*;
use efd::na;
use fmtastic::Subscript;
#[doc(no_inline)]
pub use plotters::{prelude::*, *};
use std::borrow::Cow;

mod ball;
mod dashed_line;
mod dotted_line;
pub mod fb;
pub mod sfb;

pub(crate) type PResult<T, B> = Result<T, DrawingAreaErrorKind<<B as DrawingBackend>::ErrorType>>;
pub(crate) type Canvas<B> = DrawingArea<B, coord::Shift>;

macro_rules! inner_opt {
    ($($(#[$meta:meta])+ fn $name:ident($ty:ty))+) => {$(
        $(#[$meta])+
        pub fn $name(mut self, $name: $ty) -> Self {
            self.$name = $name;
            self
        }
    )+};
}

#[inline]
pub(crate) fn to_i((x, y): (f32, f32)) -> (i32, i32) {
    (x.round() as i32, y.round() as i32)
}

#[inline]
pub(crate) fn to_f((x, y): (i32, i32)) -> (f32, f32) {
    (x as f32, y as f32)
}

// Rounding float numbers without trailing zeros
pub(crate) fn formatter(v: &f64) -> String {
    let mut s = format!("{v:.04}");
    let sub = s.trim_end_matches('0');
    s.truncate(sub.strip_suffix('.').unwrap_or(sub).len());
    if s == "-0" {
        s.remove(0);
    }
    s
}

/// The extreme values of the data.
///
/// ```
/// use four_bar::plot::ExtBound;
///
/// let data = vec![[1.], [2.], [3.]];
/// let ext = ExtBound::from_pts(&data);
/// assert_eq!(ext.min, [1.]);
/// assert_eq!(ext.max, [3.]);
/// ```
pub struct ExtBound<const N: usize> {
    /// Minimum values.
    pub min: [f64; N],
    /// Maximum values.
    pub max: [f64; N],
}

impl<const N: usize> ExtBound<N> {
    /// Create a new instance from an iterator of points.
    pub fn from_pts<'a, I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a [f64; N]>,
    {
        let init = Self {
            min: [f64::INFINITY; N],
            max: [f64::NEG_INFINITY; N],
        };
        iter.into_iter().fold(init, |mut bound, p| {
            use std::iter::zip;
            for ((p, min), max) in zip(zip(p, &mut bound.min), &mut bound.max) {
                *min = min.min(*p);
                *max = max.max(*p);
            }
            bound
        })
    }

    /// Map the extreme values to another type.
    pub fn map_to<F, R>(self, f: F) -> [R; N]
    where
        F: Fn(f64, f64) -> R,
    {
        std::array::from_fn(|i| f(self.min[i], self.max[i]))
    }

    /// Get the center of the boundary.
    pub fn center(&self) -> [f64; N] {
        std::array::from_fn(|i| (self.min[i] + self.max[i]) * 0.5)
    }

    /// Change to square boundary by the maximum range.
    ///
    /// ```
    /// use four_bar::plot::ExtBound;
    ///
    /// let ext = ExtBound { min: [0., 0.], max: [1., 2.] }.to_square(0.);
    /// assert_eq!(ext.min, [-0.5, 0.]);
    /// assert_eq!(ext.max, [1.5, 2.]);
    /// ```
    pub fn to_square(mut self, margin: f64) -> Self {
        use std::iter::zip;
        let center = self.center();
        let width = zip(&self.min, &self.max)
            .map(|(min, max)| (max - min).abs())
            .fold(0., f64::max)
            * 0.5
            * (1. + margin);
        // Extand to same range
        for ((min, max), center) in zip(zip(&mut self.min, &mut self.max), &center) {
            *min = center - width;
            *max = center + width;
        }
        self
    }
}

/// Line style.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Style {
    /// Continuous Line
    Line,
    /// Dashed Line
    DashedLine,
    /// Dotted Line
    DottedLine,
    /// Dash-dotted Line
    DashDottedLine,
    /// Dashed Line with a Marker at the starting point
    InitiatedLine,
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
    /// Style list.
    pub const LIST: [Self; 9] = [
        Self::Line,
        Self::DashedLine,
        Self::DottedLine,
        Self::DashDottedLine,
        Self::InitiatedLine,
        Self::Circle,
        Self::Triangle,
        Self::Cross,
        Self::Square,
    ];

    /// Get the style names.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Line => "Continuous Line",
            Self::DashedLine => "Dashed Line",
            Self::DottedLine => "Dotted Line",
            Self::DashDottedLine => "Dash-dotted Line",
            Self::InitiatedLine => "Initiated Line",
            Self::Circle => "Circle Marker",
            Self::Triangle => "Triangle Marker",
            Self::Cross => "Cross Marker",
            Self::Square => "Square Marker",
        }
    }

    pub(crate) fn draw<'a, DB, CT, I>(
        &self,
        chart: &mut ChartContext<'a, DB, CT>,
        line: I,
        color: ShapeStyle,
        label: &str,
        font: f64,
    ) -> PResult<(), DB>
    where
        DB: DrawingBackend + 'a,
        CT: CoordTranslate,
        CT::From: Clone + 'static,
        I: IntoIterator<Item = CT::From>,
        I::IntoIter: Clone,
    {
        let font = (font * 0.9) as i32;
        let gap = color.stroke_width as i32;
        let has_label = !label.is_empty();
        macro_rules! impl_marker {
            ($mk:expr) => {{
                let dot_size = color.stroke_width * 2;
                let color = color.stroke_width(color.stroke_width / 2);
                let mk_f = $mk; // Generic function 1
                let line = line.into_iter().map(|c| mk_f(c, dot_size, color));
                let anno = chart.draw_series(line)?;
                if has_label {
                    let mk_f = $mk; // Generic function 2
                    anno.label(label)
                        .legend(move |(x, y)| mk_f((x + font / 2, y), dot_size, color));
                }
            }};
        }
        match self {
            Self::Line => {
                let line = LineSeries::new(line, color);
                let anno = chart.draw_series(line)?;
                if has_label {
                    anno.label(label).legend(move |c| {
                        EmptyElement::at(c) + PathElement::new([(gap, 0), (font - gap, 0)], color)
                    });
                }
            }
            Self::DashedLine => {
                let series = DashedPath::new(line, 30, 15, color).series();
                let anno = chart.draw_series(series)?;
                if has_label {
                    anno.label(label).legend(move |c| {
                        EmptyElement::at(c)
                            + DashedPath::new([(gap, 0), (font - gap, 0)], 30, 15, color)
                    });
                }
            }
            Self::DottedLine => {
                let dot_size = color.stroke_width;
                let color = color.stroke_width(color.stroke_width / 2);
                let mk_f = move |c| Circle::new(c, dot_size, color);
                let series = DottedPath::new(line, 0, 20, mk_f).series();
                let anno = chart.draw_series(series)?;
                if has_label {
                    anno.label(label).legend(move |c| {
                        EmptyElement::at(c)
                            + DottedPath::new([(gap, 0), (font - gap, 0)], 0, 20, mk_f)
                    });
                }
            }
            Self::DashDottedLine => {
                let line = line.into_iter();
                let series = DashedPath::new(line.clone(), 30, 16, color).series();
                chart.draw_series(series)?;
                let dot_size = color.stroke_width / 2;
                let mk_f = move |c| Circle::new(c, dot_size, color.filled());
                let series = DottedPath::new(line, 30 + 8, 30 + 16, mk_f).series();
                let anno = chart.draw_series(series)?;
                if has_label {
                    anno.label(label).legend(move |c| {
                        let points = [(gap, 0), (font - gap, 0)];
                        EmptyElement::at(c)
                            + DashedPath::new(points, 30, 16, color)
                            + DottedPath::new(points, 30 + 8, 30 + 16, mk_f)
                    });
                }
            }
            Self::InitiatedLine => {
                let mut line = line.into_iter();
                let series = DashedPath::new(line.clone(), 30, 15, color).series();
                chart.draw_series(series)?;
                let dot_size = color.stroke_width * 2;
                let color2 = color.stroke_width(color.stroke_width / 2);
                let series = [Circle::new(line.next().unwrap(), dot_size, color2)];
                let anno = chart.draw_series(series)?;
                if has_label {
                    anno.label(label).legend(move |c| {
                        EmptyElement::at(c)
                            + DashedPath::new([(gap, 0), (font - gap, 0)], 30, 15, color)
                            + Circle::new((gap, 0), dot_size, color2)
                    });
                }
            }
            Self::Circle => impl_marker!(Circle::new),
            Self::Triangle => impl_marker!(TriangleMarker::new),
            Self::Cross => impl_marker!(Cross::new),
            Self::Square => impl_marker!(|pos, dot_size, color| {
                let r = dot_size as i32 / 2;
                EmptyElement::at(pos) + Rectangle::new([(r, r), (-r, -r)], color)
            }),
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
    #[default]
    UR,
    /// Middle Right
    MR,
    /// Lower Right
    LR,
    /// Coordinate
    #[cfg_attr(feature = "clap", clap(skip))]
    Coord(i32, i32),
}

impl LegendPos {
    /// Position list.
    pub const LIST: [Self; 10] = [
        Self::Hide,
        Self::UL,
        Self::ML,
        Self::LL,
        Self::UM,
        Self::MM,
        Self::LM,
        Self::UR,
        Self::MR,
        Self::LR,
    ];

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

/// Drawing options of a line series.
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(default)
)]
#[derive(Clone)]
pub struct LineData<'a, C: Clone> {
    /// Label of the line
    pub label: Cow<'a, str>,
    /// Line data
    pub line: Cow<'a, [C]>,
    /// Line style
    pub style: Style,
    /// Line color
    pub color: [u8; 3],
    /// Whether the line is filled
    pub filled: bool,
}

impl<'a, C: Clone> Default for LineData<'a, C> {
    fn default() -> Self {
        Self {
            label: Cow::Borrowed(""),
            line: Cow::Borrowed(&[]),
            style: Style::default(),
            color: [0; 3],
            filled: false,
        }
    }
}

impl<'a, C: Clone> LineData<'a, C> {
    pub(crate) fn color(&self) -> (RGBAColor, bool) {
        let color = RGBAColor(self.color[0], self.color[1], self.color[2], 1.);
        (color, self.filled)
    }
}

/// Option type base.
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(default)
)]
#[derive(Clone)]
pub struct FigureBase<'a, 'b, M: Clone, C: Clone> {
    /// Linkage
    pub fb: Option<Cow<'b, M>>,
    /// Line data
    pub lines: Vec<LineData<'a, C>>,
    /// Drawing options
    pub opt: Opt<'a>,
}

impl<M: Clone, C: Clone> Default for FigureBase<'_, '_, M, C> {
    fn default() -> Self {
        Self {
            fb: None,
            opt: Default::default(),
            lines: Default::default(),
        }
    }
}

impl<'a, 'b, M: Clone, C: Clone> FigureBase<'a, 'b, M, C> {
    /// Create a new instance with linkage.
    pub fn new(fb: Option<M>) -> Self {
        Self { fb: fb.map(Cow::Owned), ..Default::default() }
    }

    /// From an optional linkage setting.
    pub fn new_ref(fb: Option<&'b M>) -> Self {
        Self { fb: fb.map(Cow::Borrowed), ..Default::default() }
    }

    /// Attach linkage.
    pub fn with_fb(self, fb: M) -> Self {
        FigureBase { fb: Some(Cow::Owned(fb)), ..self }
    }

    /// Attach linkage with its reference.
    pub fn with_fb_ref(self, fb: &'b M) -> Self {
        FigureBase { fb: Some(Cow::Borrowed(fb)), ..self }
    }

    /// Remove linkage.
    pub fn remove_fb(self) -> Self {
        Self { fb: None, ..self }
    }

    /// Set the font family.
    pub fn font_family(mut self, family: impl Into<Cow<'a, str>>) -> Self {
        self.font_family = Some(family.into());
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

    /// Set the inner options.
    pub fn with_opt(self, opt: Opt<'a>) -> Self {
        Self { opt, ..self }
    }

    /// Add a line.
    pub fn add_line<S, L>(mut self, label: S, line: L, style: Style, color: RGBColor) -> Self
    where
        S: Into<Cow<'a, str>>,
        L: Into<Cow<'a, [C]>>,
    {
        self.push_line(label, line, style, color);
        self
    }

    /// Add a line with default settings.
    pub fn add_line_default<S, L>(mut self, label: S, line: L) -> Self
    where
        S: Into<Cow<'a, str>>,
        L: Into<Cow<'a, [C]>>,
    {
        self.push_line_default(label, line);
        self
    }

    /// Add a line from a [`LineData`] instance.
    pub fn add_line_data(mut self, data: LineData<'a, C>) -> Self {
        self.push_line_data(data);
        self
    }

    /// Add a line in-placed.
    pub fn push_line<S, L>(&mut self, label: S, line: L, style: Style, color: RGBColor)
    where
        S: Into<Cow<'a, str>>,
        L: Into<Cow<'a, [C]>>,
    {
        let color = ShapeStyle::from(color);
        self.push_line_data(LineData {
            label: label.into(),
            line: line.into(),
            style,
            color: [color.color.0, color.color.1, color.color.2],
            filled: color.filled,
        });
    }

    /// Add a line from a [`LineData`] instance in-placed.
    pub fn push_line_data(&mut self, data: LineData<'a, C>) {
        self.lines.push(data);
    }

    /// Add a line with default settings in-placed.
    pub fn push_line_default<S, L>(&mut self, label: S, line: L)
    where
        S: Into<Cow<'a, str>>,
        L: Into<Cow<'a, [C]>>,
    {
        self.push_line_data(LineData {
            label: label.into(),
            line: line.into(),
            ..Default::default()
        });
    }

    /// Iterate over lines.
    pub fn lines(&self) -> impl Iterator<Item = &LineData<'a, C>> {
        self.lines.iter()
    }

    /// Get a mutable reference to the lines.
    pub fn lines_mut(&mut self) -> impl Iterator<Item = &mut LineData<'a, C>> {
        self.lines.iter_mut()
    }

    /// Retain lines with a predicate.
    pub fn retain_lines(&mut self, mut f: impl FnMut(usize, &mut LineData<'a, C>) -> bool) {
        let mut i = 0;
        self.lines.retain_mut(|line| {
            i += 1;
            f(i, line)
        });
    }

    /// Swap two lines with their indices.
    pub fn swap_lines(&mut self, i: usize, j: usize) {
        self.lines.swap(i, j);
    }

    pub(crate) fn has_legend(&self) -> bool {
        self.lines.iter().any(|data| !data.label.is_empty())
    }

    #[inline]
    pub(crate) fn check_empty<B: DrawingBackend>(&self) -> PResult<(), B> {
        (!self.lines.is_empty() || self.fb.is_some())
            .then_some(())
            .ok_or(DrawingAreaErrorKind::LayoutError)
    }

    pub(crate) fn get_joints<F, const D: usize>(&self, coord_map: F) -> Option<[[f64; D]; 5]>
    where
        M: crate::mech::CurveGen<D>,
        F: Fn([f64; D]) -> na::Point2<f64>,
    {
        use std::f64::consts::TAU;
        const RES: usize = 90;

        fn angle(a: na::Point2<f64>, b: na::Point2<f64>, c: na::Point2<f64>) -> f64 {
            let ab = a - b;
            let cb = c - b;
            (ab.dot(&cb) / (ab.norm() * cb.norm())).acos()
        }

        let fb = self.fb.as_deref()?;
        let [start, end] = fb.angle_bound().to_value()?;
        let end = if end > start { end } else { end + TAU };
        let step = (end - start) / RES as f64;
        let (t, _) = (0..=RES)
            .map(|t| start + t as f64 * step)
            .filter_map(|t| Some((t, fb.pos(t)?)))
            .map(|(t, p)| {
                let [p1, p2, p3, p4, p5] = p.map(&coord_map);
                let min_angle = angle(p1, p3, p4)
                    .min(angle(p1, p3, p5))
                    .min(angle(p2, p4, p3))
                    .min(angle(p2, p4, p5));
                (t, min_angle)
            })
            .max_by(|(_, a1), (_, a2)| a1.partial_cmp(a2).unwrap())?;
        fb.pos(t)
    }

    // (stroke, dot_size)
    pub(crate) fn get_dot_size(&self) -> (u32, u32) {
        (self.stroke, (self.stroke as f32 * 1.5) as u32)
    }

    #[inline]
    fn get_family(&self) -> &str {
        const DEFAULT_FONT: &str = "Times New Roman";
        self.font_family.as_deref().unwrap_or(DEFAULT_FONT)
    }

    pub(crate) fn get_font(&self) -> TextStyle {
        (self.get_family(), self.font).into_font().color(&BLACK)
    }

    pub(crate) fn get_font3d(&self) -> TextStyle {
        (self.get_family(), self.font * 1.15)
            .into_font()
            .color(&BLACK)
    }
}

impl<'a, M: Clone, C: Clone> std::ops::Deref for FigureBase<'a, '_, M, C> {
    type Target = Opt<'a>;
    fn deref(&self) -> &Self::Target {
        &self.opt
    }
}

impl<'a, M: Clone, C: Clone> std::ops::DerefMut for FigureBase<'a, '_, M, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.opt
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
            stroke: 7,
            font: 90.,
            font_family: None,
            grid: false,
            axis: true,
            legend: LegendPos::default(),
        }
    }
}
