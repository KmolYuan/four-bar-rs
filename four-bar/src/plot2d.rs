//! The functions used to plot the 2D curve and synthesis result.
//!
//! # Single Plot Example
//!
//! ```
//! use four_bar::plot2d::*;
//! # let curves = [("", [[0.; 2]].as_slice())];
//! # let opt = None;
//! let mut buf = String::new();
//! let svg = SVGBackend::with_string(&mut buf, (800, 800));
//! plot(svg, curves, opt).unwrap();
//! ```
//!
//! # Sub-plots Example
//!
//! ```
//! use four_bar::plot2d::*;
//! # let curves = [("", [[0.; 2]].as_slice())];
//! # let opt = None;
//! let mut buf = String::new();
//! let svg = SVGBackend::with_string(&mut buf, (800, 800));
//! let (root_l, root_r) = svg.into_drawing_area().split_horizontally(800);
//! plot(root_l, curves, opt).unwrap();
//! # let curves = [("", [[0.; 2]].as_slice())];
//! # let opt = None;
//! plot(root_r, curves, opt).unwrap();
//! ```
use crate::*;
#[doc(no_inline)]
pub use plotters::{prelude::*, *};

pub(crate) type PResult<T, B> = Result<T, DrawingAreaErrorKind<<B as DrawingBackend>::ErrorType>>;
pub(crate) type Canvas<B> = DrawingArea<B, coord::Shift>;

macro_rules! inner_opt {
    ($($(#[$meta:meta])+ fn $name:ident($ty:ty))+) => {$(
        $(#[$meta])+
        pub fn $name(mut self, $name: $ty) -> Self {
            self.inner.$name = $name;
            self
        }
    )+};
}

macro_rules! impl_opt {
    ($(
        $(#[$meta:meta])+
        struct $ty:ident { $fb:ty, $coord:ty }
    )+) => {$(
        $(#[$meta])+
        #[derive(Default, Clone)]
        pub struct $ty<'a, 'b> {
            fb: Option<&'a $fb>,
            angle: Option<f64>,
            title: Option<&'b str>,
            inner: OptInner,
        }

        impl<'a> From<Option<&'a $fb>> for $ty<'a, '_> {
            fn from(opt: Option<&'a $fb>) -> Self {
                match opt {
                    Some(fb) => Self::from(fb),
                    None => Self::new(),
                }
            }
        }

        impl<'a> From<&'a $fb> for $ty<'a, '_> {
            fn from(fb: &'a $fb) -> Self {
                Self { fb: Some(fb), ..Self::default() }
            }
        }

        impl<'a, 'b> $ty<'a, 'b> {
            /// Create a default option, enables nothing.
            pub fn new() -> Self {
                Self::default()
            }

            /// Set the input angle of the linkage.
            ///
            /// If the angle value is not in the range of [`FourBar::angle_bound()`],
            /// the actual angle will be the midpoint.
            pub fn angle(self, angle: f64) -> Self {
                Self { angle: Some(angle), ..self }
            }

            /// Set the title.
            pub fn title(self, title: &'b str) -> Self {
                Self { title: Some(title), ..self }
            }

            /// Set the inner options.
            pub fn inner(self, inner: OptInner) -> Self {
                Self { inner, ..self }
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
                /// Use dot to present the curves.
                fn dot(bool)
                /// Set legend position.
                fn legend(LegendPos)
            }

            fn get_joints(&self) -> Option<[$coord; 5]> {
                let fb = self.fb.as_ref()?;
                let [start, end] = fb.angle_bound()?;
                let angle = match self.angle {
                    Some(angle) if (start..end).contains(&angle) => angle,
                    _ => start + (end - start) * 0.25,
                };
                Some(fb.pos(angle))
            }

            // (stroke, dot_size)
            fn get_stroke(&self) -> (u32, u32) {
                (self.stroke, self.stroke + 3)
            }
        }

        impl std::ops::Deref for Opt<'_, '_> {
            type Target = OptInner;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    )+};
}

pub(crate) use impl_opt;
pub(crate) use inner_opt;

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
        match pos {
            LegendPos::UL => Self::UpperLeft,
            LegendPos::ML => Self::MiddleLeft,
            LegendPos::LL => Self::LowerLeft,
            LegendPos::UM => Self::UpperMiddle,
            LegendPos::MM => Self::MiddleMiddle,
            LegendPos::LM => Self::LowerMiddle,
            LegendPos::UR => Self::UpperRight,
            LegendPos::MR => Self::MiddleRight,
            LegendPos::LR => Self::LowerRight,
            LegendPos::Coord(x, y) => Self::Coordinate(x, y),
        }
    }
}

/// 2D/3D plot option.
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(default)
)]
#[derive(Clone, PartialEq)]
pub struct OptInner {
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
    /// Use dot (marker) line
    pub dot: bool,
    /// Legend position
    pub legend: LegendPos,
}

impl Default for OptInner {
    fn default() -> Self {
        Self {
            stroke: 5,
            font: 24.,
            scale_bar: false,
            grid: false,
            axis: true,
            dot: false,
            legend: LegendPos::LR,
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

impl_opt! {
    /// Drawing option of four-bar linkage and its input angle.
    ///
    /// ```
    /// use four_bar::{plot2d::Opt, FourBar};
    /// // From linkage
    /// let opt = Opt::from(FourBar::example());
    /// // Without linkage
    /// let opt = Opt::new();
    /// ```
    struct Opt { FourBar, [f64; 2] }
}

/// Plot 2D curves and linkages.
///
/// Please see [`Opt`] for more options.
///
/// ```
/// use four_bar::{plot2d::*, FourBar};
/// let curves = [("First Curve", [[0.; 2]].as_slice())];
/// let opt = Opt::from(FourBar::example()).axis(false).scale_bar(true);
/// let mut buf = String::new();
/// let svg = SVGBackend::with_string(&mut buf, (800, 800));
/// plot(svg, curves, opt).unwrap();
/// ```
pub fn plot<'a, 'b, B, R, C, O>(root: R, curves: C, opt: O) -> PResult<(), B>
where
    B: DrawingBackend,
    Canvas<B>: From<R>,
    C: IntoIterator<Item = (&'b str, &'b [[f64; 2]])>,
    Opt<'a, 'b>: From<O>,
{
    let root = Canvas::from(root);
    root.fill(&WHITE)?;
    let opt = Opt::from(opt);
    let joints = opt.get_joints();
    let (stroke, dot_size) = opt.get_stroke();
    let curves = curves.into_iter().collect::<Vec<_>>();
    let iter = curves.iter().flat_map(|(_, curve)| curve.iter());
    let [x_min, x_max, y_min, y_max] = bounding_box(iter.chain(joints.iter().flatten()));
    let font = ("Times New Roman", opt.font).into_font().color(&BLACK);
    let font = || font.clone();
    let mut chart = ChartBuilder::on(&root);
    if let Some(title) = opt.title {
        chart.caption(title, font());
    }
    let mut chart = chart
        .set_label_area_size(LabelAreaPosition::Left, (8).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
        .margin((2).percent())
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    let mut mesh = chart.configure_mesh();
    // Draw mesh
    if !opt.grid {
        mesh.disable_mesh();
    }
    if !opt.axis {
        mesh.disable_axes();
    }
    mesh.x_label_style(font()).y_label_style(font()).draw()?;
    // Draw curve
    for (i, &(label, curve)) in curves.iter().enumerate() {
        let color = Palette99::pick(Palette99::COLORS.len() - i);
        if opt.dot {
            macro_rules! draw_dots {
                ($ty:ident) => {{
                    let line = curve
                        .iter()
                        .map(|&[x, y]| $ty::new((x, y), dot_size, &color));
                    chart
                        .draw_series(line)?
                        .label(label)
                        .legend(move |(x, y)| $ty::new((x + 10, y), dot_size, &color));
                }};
            }
            match i % 3 {
                1 => draw_dots!(TriangleMarker),
                2 => draw_dots!(Cross),
                _ => draw_dots!(Circle),
            }
        } else {
            let line = curve.iter().map(|&[x, y]| (x, y));
            chart
                .draw_series(LineSeries::new(line, color.stroke_width(stroke)))?
                .label(label)
                .legend(move |(x, y)| {
                    PathElement::new([(x, y), (x + 20, y)], color.stroke_width(stroke))
                });
        }
    }
    // Draw Linkage
    if let Some(joints @ [p0, p1, p2, p3, p4]) = joints {
        for line in [[p0, p2].as_slice(), &[p2, p4, p3, p2], &[p1, p3]] {
            let line = line.iter().map(|&[x, y]| (x, y));
            chart.draw_series(LineSeries::new(line, BLACK.stroke_width(stroke)))?;
        }
        let grounded = joints[..2].iter().map(|&[x, y]| {
            EmptyElement::at((x, y)) + TriangleMarker::new((0, 10), dot_size + 3, BLACK.filled())
        });
        chart.draw_series(grounded)?;
        let joints = joints
            .iter()
            .map(|&[x, y]| Circle::new((x, y), dot_size, BLACK.filled()));
        chart.draw_series(joints)?;
        // Draw scale bar
        if opt.scale_bar {
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
    if curves.len() > 1 {
        chart
            .configure_series_labels()
            .position(opt.legend.into())
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

/// Calculate the scale bar size.
pub fn scale_bar_size(x: f64) -> f64 {
    10f64.powi(x.log10().floor() as i32 - 1)
}
