//! The functions used to plot the 2D curve and synthesis result.
#[doc(no_inline)]
pub use super::*;

/// Drawing option of four-bar linkage and its input angle.
///
/// ```
/// use four_bar::{plot2d::Figure, FourBar};
/// // From linkage
/// let figure = Figure::from(&FourBar::example());
/// // Without linkage
/// let figure = Figure::new();
/// ```
pub type Figure<'a, 'b> = FigureBase<'a, 'b, FourBar, [f64; 2]>;

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
        .set_label_area_size(LabelAreaPosition::Bottom, (10).percent())
        .margin((4).percent())
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
    /// use four_bar::{plot::*, plot2d, FourBar};
    /// let fb = FourBar::example();
    /// let mut buf = String::new();
    /// plot2d::Figure::from(&fb)
    ///     .axis(false)
    ///     .add_line("First Curve", fb.curve(180), Style::Line, BLACK)
    ///     .plot(SVGBackend::with_string(&mut buf, (1600, 1600)))
    ///     .unwrap();
    /// ```
    pub fn plot<B, R>(&self, root: R) -> PResult<(), B>
    where
        B: DrawingBackend,
        Canvas<B>: From<R>,
    {
        self.check_empty::<B>()?;
        let root = Canvas::from(root);
        root.fill(&WHITE)?;
        let (stroke, dot_size) = self.get_dot_size();
        let joints = self.get_joints(Into::into);
        let Opt { grid, axis, legend, .. } = self.opt;
        let [x_spec, y_spec] = {
            let lines = self.lines().collect::<Vec<_>>();
            let iter = lines.iter().flat_map(|data| data.line.iter());
            area2d(iter.chain(joints.iter().flatten()), root.dim_in_pixel())
        };
        let mut chart = ChartBuilder::on(&root)
            .set_label_area_size(LabelAreaPosition::Left, (8).percent())
            .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
            .margin((4).percent())
            .build_cartesian_2d(x_spec, y_spec)?;
        let mut mesh = chart.configure_mesh();
        // Draw mesh
        if !grid {
            mesh.disable_mesh();
        }
        if !axis {
            mesh.disable_axes();
        }
        mesh.label_style(self.get_font())
            .x_label_formatter(&formatter)
            .y_label_formatter(&formatter)
            .draw()?;
        // Draw curve
        let mut has_curve = false;
        for data in self.lines() {
            let LineData { label, line, style, .. } = &*data;
            let line = line.iter().map(|&[x, y]| (x, y));
            let (color, filled) = data.color();
            let color = ShapeStyle { color, filled, stroke_width: stroke };
            style.draw(&mut chart, line, color, label, self.font as i32)?;
            has_curve |= !label.is_empty();
        }
        // Draw Linkage
        if let Some(joints @ [p1, p2, p3, p4, p5]) = joints {
            for line in [[p1, p3].as_slice(), &[p3, p5, p4, p3], &[p2, p4]] {
                let line = line.iter().map(|&[x, y]| (x, y));
                chart.draw_series(LineSeries::new(line, BLACK.stroke_width(stroke)))?;
            }
            let grounded = joints[..2].iter().map(|&[x, y]| {
                EmptyElement::at((x, y))
                    + TriangleMarker::new((0, 10), dot_size + 3, BLACK.filled())
            });
            chart.draw_series(grounded)?;
            let joints = joints.iter().enumerate().map(|(n, &[x, y])| {
                let t_style = self.get_font3d().color(&BLUE);
                EmptyElement::at((x, y))
                    + Circle::new((0, 0), dot_size, BLACK.filled())
                    + Text::new(format!("P{}", Subscript(n + 1)), (5, 5), t_style)
            });
            chart.draw_series(joints)?;
        }
        // Draw legend
        if has_curve {
            if let Some(legend) = legend.to_plotter_pos() {
                chart
                    .configure_series_labels()
                    .legend_area_size(self.font)
                    .position(legend)
                    .background_style(WHITE)
                    .border_style(BLACK)
                    .label_font(self.get_font())
                    .draw()?;
            }
        }
        Ok(())
    }
}

/// Get the 1:1 bounding box of the data, ignore the labels.
pub fn area2d<'a, I>(pts: I, area: (u32, u32)) -> [std::ops::Range<f64>; 2]
where
    I: IntoIterator<Item = &'a [f64; 2]>,
{
    let [w, h] = [area.0 as f64, area.1 as f64];
    let [[x_min, x_max], [y_min, y_max]] = ExtBound::from_pts(pts).map_to(|min, max| [min, max]);
    let dx = (x_max - x_min).abs();
    let dy = (y_max - y_min).abs();
    let x_cen = (x_min + x_max) * 0.5;
    let y_cen = (y_min + y_max) * 0.5;
    match (dx > dy, w > h, dx / dy < w / h) {
        (true, true, false) | (false, false, false) | (true, false, _) => {
            let x_r = dx * 0.5 * 1.2;
            let y_r = dx / w * h * 0.5 * 1.2;
            [x_cen - x_r..x_cen + x_r, y_cen - y_r..y_cen + y_r]
        }
        (true, true, true) | (false, false, true) | (false, true, _) => {
            let y_r = dy * 0.5 * 1.2;
            let x_r = dy / h * w * 0.5 * 1.2;
            [x_cen - x_r..x_cen + x_r, y_cen - y_r..y_cen + y_r]
        }
    }
}
