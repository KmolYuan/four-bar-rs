//! The functions used to plot the curve and synthesis result.

use crate::{curve, FourBar, Mechanism};
#[doc(no_inline)]
pub use plotters::{prelude::*, *};

type PResult<T, B> = Result<T, DrawingAreaErrorKind<<B as DrawingBackend>::ErrorType>>;

#[inline]
fn font() -> TextStyle<'static> {
    ("Times New Roman", 24).into_font().color(&BLACK)
}

/// Plot the synthesis history.
pub fn history<B>(backend: B, history: &[f64]) -> PResult<(), B>
where
    B: DrawingBackend,
{
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;
    let cap = format!(
        "Convergence Plot (Best Fitness: {:.04})",
        history[history.len() - 1]
    );
    let max_fitness = history
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(cap, font())
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

/// Plot 2D curve.
pub fn curve<B, F>(backend: B, title: &str, curves: &[(&str, &[[f64; 2]])], fb: F) -> PResult<(), B>
where
    B: DrawingBackend,
    F: Into<Option<FourBar>>,
{
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;
    let iter = curves.iter().flat_map(|(_, curve)| curve.iter());
    let fb = fb.into();
    let joints = if let Some(fb) = &fb {
        let [start, end] = fb.angle_bound().expect("invalid linkage");
        let mut joints = [[0.; 2]; 5];
        Mechanism::new(fb).apply((start + end) * 0.5, [0, 1, 2, 3, 4], &mut joints);
        Some(joints)
    } else {
        None
    };
    let [x_min, x_max, y_min, y_max] = bounding_box(iter.chain(joints.iter().flatten()));
    let mut chart = ChartBuilder::on(&root)
        .caption(title, font())
        .set_label_area_size(LabelAreaPosition::Left, (8).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
        .margin((8).percent())
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    chart
        .configure_mesh()
        .x_label_style(font())
        .y_label_style(font())
        .draw()?;
    for (i, &(label, curve)) in curves.iter().enumerate() {
        let curve = curve::get_valid_part(curve);
        let color = Palette99::pick(i);
        chart
            .draw_series(LineSeries::new(curve.iter().map(|&[x, y]| (x, y)), &color))?
            .label(label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
    }
    if let Some(joints @ [p0, p1, p2, p3, p4]) = joints {
        for [[ax, ay], [bx, by]] in [[p0, p2], [p2, p3], [p1, p3], [p2, p4], [p3, p4]] {
            chart.draw_series(LineSeries::new([(ax, ay), (bx, by)], BLACK))?;
        }
        for [x, y] in joints {
            let style = BLACK.stroke_width(0).filled();
            chart.draw_series(LineSeries::new([(x, y)], style).point_size(5))?;
        }
    }
    chart
        .configure_series_labels()
        .background_style(WHITE)
        .border_style(BLACK)
        .label_font(font())
        .draw()?;
    Ok(())
}

/// Get the bounding box of the data, ignore the labels.
pub fn bounding_box<'a>(curves: impl IntoIterator<Item = &'a [f64; 2]>) -> [f64; 4] {
    let [mut x_min, mut x_max] = [&f64::INFINITY, &-f64::INFINITY];
    let [mut y_min, mut y_max] = [&f64::INFINITY, &-f64::INFINITY];
    for [x, y] in curves {
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
