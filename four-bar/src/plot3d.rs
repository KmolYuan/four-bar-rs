//! The functions used to plot the 3D curve and synthesis result.
pub use crate::plot2d::{LegendPos, OptInner};
use crate::{plot2d::*, *};
use efd::na;
#[doc(no_inline)]
pub use plotters::{prelude::*, *};
use std::f64::consts::TAU;

/// Drawing option of spherical four-bar linkage and its input angle.
///
/// ```
/// use four_bar::{plot3d::Opt, SFourBar};
/// // From linkage
/// let opt = Opt::from(&SFourBar::example());
/// // Without linkage
/// let opt = Opt::new();
/// ```
pub type Opt<'a, 'b> = OptBase<'a, 'b, SFourBar>;

impl Opt<'_, '_> {
    fn get_sphere_center(&self) -> Option<[f64; 3]> {
        let fb = self.fb?;
        Some([fb.ox(), fb.oy(), fb.oz()])
    }

    fn get_sphere_radius(&self) -> Option<f64> {
        Some(self.fb?.r())
    }
}

/// Plot 3D spherical linkage.
///
/// Please see [`Opt`] for more options.
///
/// ```
/// use four_bar::{plot3d::*, SFourBar};
/// let curves = [("First Curve", [[0.; 3]].as_slice())];
/// let fb = SFourBar::example();
/// let opt = Opt::from(&fb).axis(false).scale_bar(true);
/// let mut buf = String::new();
/// let svg = SVGBackend::with_string(&mut buf, (800, 800));
/// plot(svg, curves, opt).unwrap();
/// ```
pub fn plot<'a, 'b, B, R, C, O>(root: R, curves: C, opt: O) -> PResult<(), B>
where
    B: DrawingBackend,
    Canvas<B>: From<R>,
    C: IntoIterator<Item = (&'b str, &'b [[f64; 3]])>,
    Opt<'a, 'b>: From<O>,
{
    let root = Canvas::from(root);
    root.fill(&WHITE)?;
    let opt = Opt::from(opt);
    let joints = opt.get_joints();
    let sc = na::Vector3::from(opt.get_sphere_center().unwrap_or_default());
    let sr = opt.get_sphere_radius().unwrap_or(1.);
    debug_assert!(sr > 0.);
    let (stroke, dot_size) = opt.get_stroke();
    let curves = curves.into_iter().collect::<Vec<_>>();
    let font = ("Times New Roman", opt.font).into_font().color(&BLACK);
    let font = || font.clone();
    let mut chart = ChartBuilder::on(&root);
    if let Some(title) = opt.get_title() {
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
    if opt.axis {
        chart
            .configure_axes()
            .light_grid_style(BLACK.mix(0.15))
            .label_style(font())
            .max_light_lines(3)
            .draw()?;
    }
    // Draw grid
    if opt.grid {
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
    if opt.scale_bar {
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
    for (i, &(label, curve)) in curves.iter().enumerate() {
        let color = Palette99::pick(Palette99::COLORS.len() - i);
        if opt.dot {
            macro_rules! draw_dots {
                ($ty:ident) => {{
                    let line = curve
                        .iter()
                        .map(|&[x, y, z]| $ty::new((x, y, z), dot_size, &color));
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
            let line = curve.iter().map(|&[x, y, z]| (x, y, z));
            chart
                .draw_series(LineSeries::new(line, color.stroke_width(stroke)))?
                .label(label)
                .legend(move |(x, y)| {
                    PathElement::new([(x, y), (x + 20, y)], color.stroke_width(stroke))
                });
        }
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
                        let p = na::UnitQuaternion::from_scaled_axis(axis * i as f64 * step) * a;
                        [sc.x + p.x, sc.y + p.y, sc.z + p.z]
                    })
                })
                .map(|[x, y, z]| (x, y, z));
            chart.draw_series(LineSeries::new(line, BLACK.stroke_width(stroke)))?;
        }
        let grounded = joints[..2].iter().map(|&[x, y, z]| {
            EmptyElement::at((x, y, z)) + TriangleMarker::new((0, 10), dot_size + 3, BLACK.filled())
        });
        chart.draw_series(grounded)?;
        let joints = joints
            .iter()
            .map(|&[x, y, z]| Circle::new((x, y, z), dot_size, BLACK.filled()));
        chart.draw_series(joints)?;
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
