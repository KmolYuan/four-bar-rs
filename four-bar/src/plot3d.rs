//! The functions used to plot the 3D curve and synthesis result.
pub use crate::plot2d::{LegendPos, Opt, Style};
use crate::{plot2d::*, *};
use efd::na;
#[doc(no_inline)]
pub use plotters::{prelude::*, *};
use std::f64::consts::TAU;

/// Drawing option of spherical four-bar linkage and its input angle.
///
/// Please see [`Figure::plot()`] for more information.
pub type Figure<'a, 'b> = FigureBase<'a, 'b, SFourBar, 3>;

impl Figure<'_, '_> {
    fn get_sphere_center(&self) -> Option<[f64; 3]> {
        let fb = self.fb?;
        Some([fb.ox(), fb.oy(), fb.oz()])
    }

    fn get_sphere_radius(&self) -> Option<f64> {
        Some(self.fb?.r())
    }

    /// Plot 3D spherical linkage.
    ///
    /// Please see [`Opt`] for more options.
    ///
    /// ```
    /// use four_bar::{plot3d::*, SFourBar};
    /// let fb = SFourBar::example();
    /// let mut buf = String::new();
    /// Figure::from(&fb)
    ///     .axis(false)
    ///     .scale_bar(true)
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
        let font = self.get_font();
        let sc = na::Vector3::from(self.get_sphere_center().unwrap_or_default());
        let sr = self.get_sphere_radius().unwrap_or(1.);
        debug_assert!(sr > 0.);
        let Self {
            lines,
            opt: Opt { title, scale_bar, grid, axis, legend, .. },
            ..
        } = self;
        let font = || font.clone();
        let mut chart = ChartBuilder::on(&root);
        if let Some(title) = title {
            chart.caption(title, font());
        }
        let mut chart = chart
            .set_label_area_size(LabelAreaPosition::Left, (8).percent())
            .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
            .margin((8).percent())
            .build_cartesian_3d(
                sc.y - sr..sc.y + sr,
                sc.z - sr..sc.z + sr,
                sc.x - sr..sc.x + sr,
            )?;
        chart.with_projection(|mut pb| {
            pb.yaw = 45f64.to_radians();
            pb.scale = 0.9;
            pb.into_matrix()
        });
        if *axis {
            chart
                .configure_axes()
                .light_grid_style(BLACK.mix(0.15))
                .label_style(font())
                .max_light_lines(3)
                .draw()?;
        }
        // Draw grid
        if *grid {
            let t = (0..=500).map(|t| t as f64 / 500. * TAU);
            let z = t.clone().map(|t| sr * t.cos());
            let y = t.map(|t| sr * t.sin());
            const N: usize = 96;
            for i in 0..N {
                let phi = i as f64 / N as f64 * TAU;
                let x = z.clone().map(|z| z * phi.sin());
                let z = z.clone().map(|z| z * phi.cos());
                let iter = x
                    .zip(y.clone())
                    .zip(z)
                    .map(|((x, y), z)| (sc.x + x, sc.y + y, sc.z + z));
                chart.draw_series(LineSeries::new(iter, BLACK.mix(0.1)))?;
            }
        }
        // Draw scale bar
        if *scale_bar {
            let scale_bar = scale_bar_size(sr);
            for (p, color) in [
                ([scale_bar, 0., 0.], RED),
                ([0., scale_bar, 0.], BLUE),
                ([0., 0., scale_bar], GREEN),
            ] {
                let p = na::Vector3::from(p) + sc;
                chart.draw_series(LineSeries::new(
                    [(sc.x, sc.y, sc.z), (p.x, p.y, p.z)],
                    color.stroke_width(stroke),
                ))?;
            }
        }
        // Draw curves
        for (label, line, style, color) in lines {
            macro_rules! marker {
                ($mk:ident) => {{
                    let line = line
                        .iter()
                        .map(|&[x, y, z]| $mk::new((x, y, z), dot_size, color));
                    let anno = chart.draw_series(line)?;
                    if !label.is_empty() {
                        anno.label(label.as_ref())
                            .legend(move |(x, y)| $mk::new((x, y), dot_size, color));
                    }
                }};
            }
            match style {
                Style::Line => {
                    let line = line.iter().map(|&[x, y, z]| (x, y, z));
                    let anno =
                        chart.draw_series(LineSeries::new(line, color.stroke_width(stroke)))?;
                    if !label.is_empty() {
                        anno.label(label.as_ref()).legend(move |(x, y)| {
                            PathElement::new([(x, y), (x + 20, y)], color.stroke_width(stroke))
                        });
                    }
                }
                Style::Triangle => marker!(TriangleMarker),
                Style::Cross => marker!(Cross),
                Style::Circle => marker!(Circle),
                Style::Square => {
                    let r = dot_size as i32 / 2;
                    let line = line.iter().map(|&[x, y, z]| {
                        EmptyElement::at((x, y, z)) + Rectangle::new([(r, r), (-r, -r)], color)
                    });
                    let anno = chart.draw_series(line)?;
                    if !label.is_empty() {
                        anno.label(label.as_ref()).legend(move |(x, y)| {
                            Rectangle::new([(x + r, y + r), (x - r, y - r)], color)
                        });
                    }
                }
            };
        }
        // Draw linkage
        if let Some(joints @ [p0, p1, p2, p3, p4]) = joints {
            for line in [[p0, p2].as_slice(), &[p2, p4, p3, p2], &[p1, p3]] {
                let line = line
                    .windows(2)
                    .flat_map(|w| {
                        let sc = na::Point3::from(sc);
                        let a = na::Point3::from(w[0]) - sc;
                        let b = na::Point3::from(w[1]) - sc;
                        let axis = a.cross(&b).normalize();
                        let angle = a.normalize().dot(&b.normalize()).acos();
                        const N: usize = 150;
                        let step = angle / N as f64;
                        (0..=N).map(move |i| {
                            let p =
                                na::UnitQuaternion::from_scaled_axis(axis * i as f64 * step) * a;
                            [sc.x + p.x, sc.y + p.y, sc.z + p.z]
                        })
                    })
                    .map(|[x, y, z]| (x, y, z));
                chart.draw_series(LineSeries::new(line, BLACK.stroke_width(stroke)))?;
            }
            let grounded = joints[..2].iter().map(|&[x, y, z]| {
                EmptyElement::at((x, y, z))
                    + TriangleMarker::new((0, 10), dot_size + 3, BLACK.filled())
            });
            chart.draw_series(grounded)?;
            let joints = joints
                .iter()
                .map(|&[x, y, z]| Circle::new((x, y, z), dot_size, BLACK.filled()));
            chart.draw_series(joints)?;
        }
        if let Some(legend) = *legend {
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
