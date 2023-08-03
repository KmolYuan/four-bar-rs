//! Plot utitlities.
//!
//! Please see [`crate::plot2d::Figure`] and [`crate::plot3d::Figure`] for more
//! information.
//!
//! # Single Plot Example
//!
//! ```
//! use four_bar::{plot::*, plot2d};
//!
//! let fig = plot2d::Figure::new().add_line("", vec![[0.; 2], [1.; 2]], Style::Line, BLACK);
//! let mut buf = String::new();
//! let svg = SVGBackend::with_string(&mut buf, (800, 800));
//! fig.plot(svg).unwrap();
//! ```
//!
//! # Sub-plots Example
//!
//! ```
//! use four_bar::{plot::*, plot2d};
//!
//! let fig = plot2d::Figure::new().add_line("", vec![[0.; 2], [1.; 2]], Style::Line, BLACK);
//! let mut buf = String::new();
//! let svg = SVGBackend::with_string(&mut buf, (1600, 800));
//! let (root_l, root_r) = svg.into_drawing_area().split_horizontally(800);
//! fig.plot(root_l).unwrap();
//! fig.plot(root_r).unwrap();
//! ```
use crate::*;
use plotters::element::DashedPathElement;
#[doc(no_inline)]
pub use plotters::{prelude::*, *};
use std::{borrow::Cow, rc::Rc};

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

// Rounding float numbers
pub(crate) fn formatter(v: &f64) -> String {
    let mut s = format!("{v:.02}");
    let sub = s.trim_end_matches('0');
    s.truncate(sub.strip_suffix('.').unwrap_or(sub).len());
    s
}

/// The extreme values of the data.
///
/// ```
/// use four_bar::plot::ExtBound;
///
/// let data = vec![[1.], [2.], [3.]];
/// let ext = ExtBound::from_iter(&data);
/// assert_eq!(ext.min, [1.]);
/// assert_eq!(ext.max, [3.]);
/// ```
pub struct ExtBound<const N: usize> {
    /// Minimum values.
    pub min: [f64; N],
    /// Maximum values.
    pub max: [f64; N],
}

impl<'a, const N: usize> FromIterator<&'a [f64; N]> for ExtBound<N> {
    fn from_iter<I: IntoIterator<Item = &'a [f64; N]>>(iter: I) -> Self {
        let init = Self {
            min: [f64::INFINITY; N],
            max: [f64::NEG_INFINITY; N],
        };
        iter.into_iter().fold(init, |mut bound, p| {
            p.iter()
                .zip(&mut bound.min)
                .zip(&mut bound.max)
                .for_each(|((p, min), max)| {
                    *min = min.min(*p);
                    *max = max.max(*p);
                });
            bound
        })
    }
}

impl<const N: usize> ExtBound<N> {
    /// Map the extreme values to another type.
    pub fn map_to<F, R>(self, f: F) -> [R; N]
    where
        F: Fn(f64, f64) -> R,
        [R; N]: Default,
    {
        let Self { min, max } = self;
        min.into_iter()
            .zip(max)
            .map(|(min, max)| f(min, max))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_default()
    }

    /// Get the center of the boundary.
    pub fn center(&self) -> [f64; N] {
        self.min
            .iter()
            .zip(&self.max)
            .map(|(min, max)| (max - min) / 2.)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    /// Change to square boundary by the maximum range.
    ///
    /// ```
    /// use four_bar::plot::ExtBound;
    ///
    /// let ext = ExtBound { min: [0., 0.], max: [1., 2.] }.to_square();
    /// assert_eq!(ext.min, [-0.5, 0.]);
    /// assert_eq!(ext.max, [1.5, 2.]);
    /// ```
    pub fn to_square(mut self) -> Self {
        let center = self.center();
        let width = self
            .min
            .iter()
            .zip(&self.max)
            .map(|(min, max)| (max - min))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        // Extand to same range
        self.min
            .iter_mut()
            .zip(&mut self.max)
            .zip(&center)
            .for_each(|((min, max), center)| {
                *min = center - width / 2.;
                *max = center + width / 2.;
            });
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
    /// Circle Marker
    #[default]
    Circle,
    /// Dot Marker
    Dot,
    /// Triangle Marker
    Triangle,
    /// Cross Marker
    Cross,
    /// Square Marker
    Square,
}

impl Style {
    /// Style list.
    pub const LIST: [Self; 7] = [
        Self::Line,
        Self::DashedLine,
        Self::Circle,
        Self::Dot,
        Self::Triangle,
        Self::Cross,
        Self::Square,
    ];

    /// Get the style names.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Line => "Line",
            Self::DashedLine => "Dashed Line",
            Self::Circle => "Circle",
            Self::Dot => "Dot",
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
        I: Iterator<Item = CT::From> + Clone,
    {
        let color = color.stroke_width(color.stroke_width + 2);
        let dot_size = color.stroke_width + 5;
        let has_label = !label.is_empty();
        macro_rules! impl_marker {
            ($mk:ident) => {{
                let line = line.into_iter().map(|c| $mk::new(c, dot_size, color));
                let anno = chart.draw_series(line)?;
                if has_label {
                    anno.label(label)
                        .legend(move |(x, y)| $mk::new((x + 10, y), dot_size, color));
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
            Self::DashedLine => {
                let series = DashedLineSeries::new(line, 10, 5, color);
                let anno = chart.draw_series(series)?;
                if has_label {
                    anno.label(label).legend(move |(x, y)| {
                        DashedPathElement::new([(x, y), (x + 20, y)], 10, 5, color)
                    });
                }
            }
            Self::Circle => impl_marker!(Circle),
            Self::Dot => {
                let color = color.filled();
                let dot_size = dot_size - 4;
                let line = line.into_iter().map(|c| Circle::new(c, dot_size, color));
                let anno = chart.draw_series(line)?;
                if has_label {
                    anno.label(label).legend(move |c| {
                        EmptyElement::at(c)
                            + Circle::new((0, 0), dot_size, color)
                            + Circle::new((10, 0), dot_size, color)
                            + Circle::new((20, 0), dot_size, color)
                    });
                }
            }
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

    /// Set the inner options.
    pub fn with_opt(self, opt: Opt<'a>) -> Self {
        Self { opt, ..self }
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

    #[inline]
    pub(crate) fn check_empty<B: DrawingBackend>(&self) -> PResult<(), B> {
        (!self.lines.is_empty() || self.fb.is_some())
            .then_some(())
            .ok_or(DrawingAreaErrorKind::LayoutError)
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
