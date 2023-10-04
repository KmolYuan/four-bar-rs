//! The functions used to plot the 3D curve and synthesis result.
#[doc(no_inline)]
pub use super::*;
use efd::na;
use full_palette::GREY_600;
#[doc(no_inline)]
pub use plotters::{prelude::*, *};

/// Drawing option of spherical four-bar linkage and its input angle.
///
/// Please see [`Figure::plot()`] for more information.
pub type Figure<'a, 'b> = FigureBase<'a, 'b, SFourBar, [f64; 3]>;

impl Figure<'_, '_> {
    fn get_sphere_center_radius(&self) -> Option<(na::Point3<f64>, f64)> {
        let fb = &self.fb.as_deref()?.unnorm;
        Some((na::Point3::new(fb.ox, fb.oy, fb.oz), fb.r))
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
        let sphere = self
            .get_sphere_center_radius()
            .map(|(sc, r)| (sc, r, self.get_joints()));
        let [x_spec, y_spec, z_spec] = if let Some((sc, r, _)) = &sphere {
            debug_assert!(*r > 0.);
            [sc.x - r..sc.x + r, sc.y - r..sc.y + r, sc.z - r..sc.z + r]
        } else {
            let lines = self.lines().collect::<Vec<_>>();
            area3d(lines.iter().flat_map(|data| data.line.iter()))
        };
        let Opt { grid, axis, legend, .. } = self.opt;
        let mut chart = ChartBuilder::on(&root)
            .set_label_area_size(LabelAreaPosition::Left, (8).percent())
            .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
            .margin((2).percent())
            .margin_left((11).percent())
            .build_cartesian_3d(x_spec, y_spec, z_spec)?;
        let yaw = std::f64::consts::FRAC_PI_4;
        chart.with_projection(|mut pb| {
            pb.yaw = yaw;
            pb.scale = 0.9;
            pb.into_matrix()
        });
        if axis {
            let mut axes = chart.configure_axes();
            if !grid {
                axes.max_light_lines(0);
            }
            axes.light_grid_style(LIGHTGRAY)
                .label_style(self.get_font3d())
                .axis_panel_style(TRANSPARENT)
                .x_labels(4)
                .z_labels(4)
                .x_formatter(&formatter)
                .y_formatter(&formatter)
                .z_formatter(&formatter)
                .draw()?;
        }
        // Draw grid
        if let Some((sc, r, _)) = &sphere {
            let p = (sc.x, sc.y + *r, sc.z);
            chart.draw_series(Ball::new((sc.x, sc.y, sc.z), p, LIGHTGRAY.filled()).series())?;
        }
        // Draw layer 1: Draw linkage in the back of the sphere
        let mut link_front = Vec::new();
        let mut grounded_front = Vec::new();
        let mut joints_front = Vec::new();
        if let Some((sc, _, Some(joints))) = sphere {
            let [p0, p1, p2, p3, p4] = joints;
            for line in [[p0, p2].as_slice(), &[p2, p4, p3, p2], &[p1, p3]] {
                let mut line = line.windows(2).flat_map(|w| {
                    let a = na::Point3::from(w[0]) - sc;
                    let b = na::Point3::from(w[1]) - sc;
                    let axis = a.cross(&b).normalize();
                    let angle = a.normalize().dot(&b.normalize()).acos();
                    const N: usize = 150;
                    let step = angle / N as f64;
                    (0..=N).map(move |i| {
                        let p = na::UnitQuaternion::from_scaled_axis(axis * i as f64 * step) * a;
                        [sc.x + p.x, sc.y + p.y, sc.z + p.z]
                    })
                });
                let mut last_pt = None;
                loop {
                    let is_front = std::cell::OnceCell::new();
                    let mut line = line
                        .by_ref()
                        .take_while(|&[x, y, z]| {
                            let stat = is_front_of_sphere(sc, na::Point3::new(x, y, z), yaw);
                            *is_front.get_or_init(|| stat) == stat
                        })
                        .map(|[x, y, z]| (x, y, z))
                        .collect::<Vec<_>>();
                    if line.is_empty() {
                        break;
                    }
                    if let Some(pre_pt) = last_pt {
                        line.insert(0, pre_pt);
                    }
                    last_pt = line.last().copied();
                    let is_front = is_front.into_inner().unwrap();
                    let color = if is_front { BLACK } else { GREY_600 };
                    let line = LineSeries::new(line, color.stroke_width(stroke));
                    if is_front {
                        link_front.push(line);
                    } else {
                        chart.draw_series(line)?;
                    }
                }
            }
            for &[x, y, z] in &joints[..2] {
                let is_front = is_front_of_sphere(sc, na::Point3::new(x, y, z), yaw);
                let style = if is_front { BLACK } else { GREY_600 }.filled();
                let grounded = EmptyElement::at((x, y, z))
                    + TriangleMarker::new((0, dot_size as i32), dot_size * 2, style);
                if is_front {
                    grounded_front.push(grounded);
                } else {
                    chart.draw_series([grounded])?;
                }
            }
            for (n, [x, y, z]) in joints.into_iter().enumerate() {
                let is_front = is_front_of_sphere(sc, na::Point3::new(x, y, z), yaw);
                let color = if is_front { BLACK } else { GREY_600 }.to_rgba();
                let style = ShapeStyle { color, filled: n == 4, stroke_width: stroke };
                let joint = Circle::new((x, y, z), dot_size, style);
                if is_front {
                    joints_front.push(joint);
                } else {
                    chart.draw_series([joint])?;
                }
            }
        }
        // Draw layer 2: Draw curves
        for data in self.lines() {
            let LineData { label, line, style, .. } = &*data;
            let line = line.iter().map(|&c| c.into());
            let (color, filled) = data.color();
            let color = ShapeStyle { color, filled, stroke_width: stroke };
            style.draw(&mut chart, line, color, label, self.font as i32)?;
        }
        // Draw layer 3: Draw linkage in the front of the sphere
        for line in link_front {
            chart.draw_series(line)?;
        }
        chart.draw_series(grounded_front)?;
        chart.draw_series(joints_front)?;
        // Draw legend
        if let Some(legend) = legend.to_plotter_pos() {
            chart
                .configure_series_labels()
                .legend_area_size(self.font)
                .position(legend)
                .background_style(WHITE)
                .border_style(BLACK)
                .label_font(self.get_font3d())
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

/// Check the point is in front of the sphere.
pub fn is_front_of_sphere(sc: na::Point3<f64>, pt: na::Point3<f64>, yaw: f64) -> bool {
    let dir = na::Vector3::new(yaw.sin(), 0., yaw.cos());
    let v = pt - sc;
    (v.dot(&dir) / (v.norm() * dir.norm())).acos() < std::f64::consts::FRAC_PI_2
}
