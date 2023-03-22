//! The functions used to plot the 3D curve and synthesis result.
use crate::{plot2d::*, *};
#[doc(no_inline)]
pub use plotters::{prelude::*, *};
use std::f64::consts::TAU;

impl_opt! {
    /// Drawing option of spherical four-bar linkage and its input angle.
    struct Opt { SFourBar, [f64; 3] }
}

/// Plot 3D spherical linkage.
///
/// Please see [`Opt`] for more options.
///
/// ```
/// use four_bar::plot3d::*;
/// let curves = [("First Curve", [[0.; 3]].as_slice())];
/// let opt = Opt::new().axis(false).scale_bar(10.);
/// let mut buf = String::new();
/// let svg = SVGBackend::with_string(&mut buf, (800, 800));
/// plot(svg, 1., curves, opt).unwrap();
/// ```
pub fn plot<'a, B, R, C, O>(root: R, sr: f64, curves: C, opt: O) -> PResult<(), B>
where
    B: DrawingBackend,
    Canvas<B>: From<R>,
    C: IntoIterator<Item = (&'a str, &'a [[f64; 3]])>,
    Opt<'a>: From<O>,
{
    debug_assert!(sr > 0.);
    let root = Canvas::from(root);
    root.fill(&WHITE)?;
    let opt = Opt::from(opt);
    let joints = opt.joints();
    let curves = curves.into_iter().collect::<Vec<_>>();
    let font = ("Times New Roman", opt.font).into_font().color(&BLACK);
    let font = || font.clone();
    let mut chart = ChartBuilder::on(&root);
    if let Some(title) = opt.title {
        chart.caption(title, font());
    }
    let mut chart = chart
        .set_label_area_size(LabelAreaPosition::Left, (8).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
        .margin((8).percent())
        .build_cartesian_3d(-sr..sr, -sr..sr, -sr..sr)?;
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
            let iter = x.zip(y.clone()).zip(z).map(|((x, y), z)| (x, y, z));
            chart.draw_series(LineSeries::new(iter, BLACK.mix(0.1)))?;
        }
    }
    // Draw scale bar
    for (p, color) in [
        ((opt.scale_bar, 0., 0.), RED),
        ((0., opt.scale_bar, 0.), GREEN),
        ((0., 0., opt.scale_bar), BLUE),
    ] {
        chart.draw_series(LineSeries::new([(0., 0., 0.), p], color.stroke_width(5)))?;
    }
    // Draw curves
    for (i, &(label, curve)) in curves.iter().enumerate() {
        let color = Palette99::pick(i);
        let stroke = opt.stroke;
        if opt.dot {
            if i % 2 == 1 {
                let series = curve
                    .iter()
                    .map(|&[x, y, z]| Circle::new((x, z, y), stroke, &color));
                chart
                    .draw_series(series)?
                    .label(label)
                    .legend(move |(x, y)| Circle::new((x + 10, y), stroke, &color));
            } else {
                let series = curve
                    .iter()
                    .map(|&[x, y, z]| TriangleMarker::new((x, z, y), stroke, &color));
                chart
                    .draw_series(series)?
                    .label(label)
                    .legend(move |(x, y)| TriangleMarker::new((x + 10, y), stroke, &color));
            };
        } else {
            chart
                .draw_series(LineSeries::new(
                    curve.iter().map(|&[x, y, z]| (x, z, y)),
                    color.stroke_width(stroke),
                ))?
                .label(label)
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(stroke))
                });
        }
    }
    // Draw linkage
    if let Some(joints @ [p0, p1, p2, p3, p4]) = joints {
        let to_sc = |[x, y, z]: [f64; 3]| [x.hypot(y).atan2(z), y.atan2(x)];
        let to_cc = |(theta, psi): (f64, f64)| {
            let x = sr * theta.sin() * psi.cos();
            let y = sr * theta.sin() * psi.sin();
            let z = sr * theta.cos();
            (x, y, z)
        };
        let linspace = |start: f64, end: f64| {
            const N: usize = 150;
            let step = (end - start) / N as f64;
            (0..N).map(move |i| start + i as f64 * step)
        };
        let link = |a: [f64; 3], b: [f64; 3]| {
            let [[theta1, psi1], [theta2, psi2]] = [to_sc(a), to_sc(b)];
            linspace(theta1, theta2)
                .zip(linspace(psi1, psi2))
                .map(to_cc)
        };
        for line in [[p0, p2].as_slice(), &[p2, p4, p3, p2], &[p1, p3]] {
            chart.draw_series(LineSeries::new(
                line.windows(2)
                    .flat_map(|w| link(w[0], w[1]))
                    .map(|(x, y, z)| (x, z, y)),
                BLACK.stroke_width(3),
            ))?;
        }
        let joints_iter = joints
            .iter()
            .map(|&[x, y, z]| Circle::new((x, z, y), 5, BLACK.filled()));
        chart.draw_series(joints_iter)?;
        let grounded = joints[..2].iter().map(|&[x, y, z]| {
            let r = 3e-2;
            Cubiod::new(
                [(x - r, z - r, y - r), (x + r, z + r, y + r)],
                BLACK.mix(0.2),
                BLACK.filled(),
            )
        });
        chart.draw_series(grounded)?;
    }
    if curves.len() > 1 {
        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::LowerRight)
            .background_style(WHITE)
            .border_style(BLACK)
            .label_font(font())
            .draw()?;
    }
    Ok(())
}
