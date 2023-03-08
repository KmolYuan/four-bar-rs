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
pub fn plot<'a, B, C, O>(backend: B, sr: f64, curves: C, opt: O) -> PResult<(), B>
where
    B: DrawingBackend,
    C: IntoIterator<Item = (&'a str, &'a [[f64; 3]])>,
    O: Into<Opt<'a>>,
{
    debug_assert!(sr > 0.);
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;
    let opt = opt.into();
    let joints = opt.joints();
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
        pb.yaw = 0.9;
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
    // Draw the sphere
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
        ((0.1, 0., 0.), RED),
        ((0., 0.1, 0.), GREEN),
        ((0., 0., 0.1), BLUE),
    ] {
        chart.draw_series(LineSeries::new([(0., 0., 0.), p], color.stroke_width(5)))?;
    }
    // Draw the curves
    for (i, (label, curve)) in curves.into_iter().enumerate() {
        let color = Palette99::pick(i);
        let stroke = opt.stroke;
        if opt.dot {
            if i % 2 == 1 {
                let series = curve
                    .iter()
                    .map(|&[x, y, z]| Circle::new((x, y, z), stroke, &color));
                chart
                    .draw_series(series)?
                    .label(label)
                    .legend(move |(x, y)| Circle::new((x + 10, y), stroke, &color));
            } else {
                let series = curve
                    .iter()
                    .map(|&[x, y, z]| TriangleMarker::new((x, y, z), stroke, &color));
                chart
                    .draw_series(series)?
                    .label(label)
                    .legend(move |(x, y)| TriangleMarker::new((x + 10, y), stroke, &color));
            };
        } else {
            chart
                .draw_series(LineSeries::new(
                    curve.iter().map(|&[x, y, z]| (x, y, z)),
                    color.stroke_width(stroke),
                ))?
                .label(label)
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(stroke))
                });
        }
    }
    if let Some(_joints) = joints {
        // TODO
    }
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .background_style(WHITE)
        .border_style(BLACK)
        .label_font(font())
        .draw()?;
    Ok(())
}
