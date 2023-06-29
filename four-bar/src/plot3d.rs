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
        let sc = na::Vector3::from(self.get_sphere_center().unwrap_or_default());
        let sr = self.get_sphere_radius().unwrap_or(1.);
        debug_assert!(sr > 0.);
        let Opt { grid, axis, legend, .. } = self.opt;
        let mut chart = ChartBuilder::on(&root)
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
        if axis {
            chart
                .configure_axes()
                .light_grid_style(BLACK.mix(0.15))
                .label_style(self.get_axis_font())
                .max_light_lines(3)
                .x_formatter(&formatter)
                .y_formatter(&formatter)
                .z_formatter(&formatter)
                .draw()?;
        }
        // Draw grid
        if grid {
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
        // Draw curves
        for (label, line, style, color) in self.lines() {
            let line = line.iter().map(|&[x, y, z]| (x, y, z));
            style.draw(&mut chart, line, *color, label)?;
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
