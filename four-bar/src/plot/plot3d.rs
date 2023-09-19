//! The functions used to plot the 3D curve and synthesis result.
#[doc(no_inline)]
pub use crate::plot::*;
use efd::na;
#[doc(no_inline)]
pub use plotters::{prelude::*, *};
use std::f64::consts::TAU;

/// Drawing option of spherical four-bar linkage and its input angle.
///
/// Please see [`Figure::plot()`] for more information.
pub type Figure<'a, 'b> = FigureBase<'a, 'b, crate::SFourBar, 3>;

impl Figure<'_, '_> {
    fn get_sphere_center_radius(&self) -> Option<(na::Vector3<f64>, f64)> {
        let fb = self.fb?;
        Some((na::Vector3::new(fb.ox(), fb.oy(), fb.oz()), fb.r()))
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
        self.check_empty::<B>()?;
        let root = Canvas::from(root);
        root.fill(&WHITE)?;
        let (stroke, dot_size) = self.get_dot_size();
        let sphere = self.get_joints().zip(self.get_sphere_center_radius());
        let [x_spec, y_spec, z_spec] = if let Some((_, (sc, r))) = sphere {
            debug_assert!(r > 0.);
            [sc.x - r..sc.x + r, sc.y - r..sc.y + r, sc.z - r..sc.z + r]
        } else {
            area3d(self.lines().flat_map(|(_, curve, ..)| curve.iter()))
        };
        let reflect = [x_spec.start, y_spec.start, z_spec.start];
        let Opt { grid, axis, legend, .. } = self.opt;
        let mut chart = ChartBuilder::on(&root)
            .set_label_area_size(LabelAreaPosition::Left, (8).percent())
            .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
            .margin((2).percent())
            .margin_left((11).percent())
            .build_cartesian_3d(x_spec, y_spec, z_spec)?;
        chart.with_projection(|mut pb| {
            pb.yaw = 45f64.to_radians();
            pb.scale = 0.9;
            pb.into_matrix()
        });
        if axis {
            let mut axes = chart.configure_axes();
            if !grid {
                axes.max_light_lines(0);
            }
            axes.light_grid_style(LIGHTGRAY)
                .label_style(self.get_font())
                .axis_panel_style(TRANSPARENT)
                .x_labels(4)
                .z_labels(4)
                .x_formatter(&formatter)
                .y_formatter(&formatter)
                .z_formatter(&formatter)
                .draw()?;
        }
        // Draw grid
        if let Some((_, (sc, r))) = &sphere {
            let t = (0..=500).map(|t| t as f64 / 500. * TAU);
            let z = t.clone().map(|t| r * t.cos());
            let y = t.map(|t| r * t.sin());
            const N: usize = 90;
            for i in 0..N {
                let phi = i as f64 / N as f64 * TAU;
                let x = z.clone().map(|z| z * phi.sin());
                let z = z.clone().map(|z| z * phi.cos());
                let iter = x
                    .zip(y.clone())
                    .zip(z)
                    .map(|((x, y), z)| (sc.x + x, sc.y + y, sc.z + z));
                chart.draw_series(LineSeries::new(iter, LIGHTGRAY))?;
            }
        } else {
            // Draw reflections
            for (_, line, style, color) in self.lines() {
                for (i, b) in reflect.iter().enumerate() {
                    let line = line.iter().cloned().map(|mut c| {
                        c[i] = *b;
                        c.into()
                    });
                    style.draw(&mut chart, line, *color, "")?;
                }
            }
        }
        // Draw curves
        for (label, line, style, color) in self.lines() {
            let line = line.iter().map(|&c| c.into());
            style.draw(&mut chart, line, *color, label)?;
        }
        // Draw linkage
        if let Some((joints @ [p0, p1, p2, p3, p4], (sc, _))) = sphere {
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
            let grounded = joints[..2].iter().map(|&c| {
                EmptyElement::at(c.into())
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

/// Get the area of a set of points in 3D.
pub fn area3d<'a, I>(pts: I) -> [std::ops::Range<f64>; 3]
where
    I: IntoIterator<Item = &'a [f64; 3]>,
{
    ExtBound::from_pts(pts)
        .to_square(0.2)
        .map_to(|min, max| min..max)
}
