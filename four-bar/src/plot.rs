//! The functions used to plot the curve and synthesis result.

use self::prelude::*;
pub use plotters::*;
use std::error::Error;

fn font() -> TextStyle<'static> {
    ("Times New Roman", 24).into_font().color(&BLACK)
}

/// Plot the synthesis history.
pub fn plot_history<B>(backend: B, history: &[f64], fitness: f64) -> Result<(), Box<dyn Error>>
where
    B: DrawingBackend + 'static,
{
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;
    let cap = format!("Convergence Plot (Best Fitness: {:.04})", fitness);
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
    let history = history.iter().enumerate().map(|(i, x)| (i, *x));
    chart.draw_series(LineSeries::new(history, &BLUE))?;
    Ok(())
}

/// Plot 2D curve.
pub fn plot_curve<B>(
    backend: B,
    title: &str,
    curves: &[(&str, &[[f64; 2]])],
) -> Result<(), Box<dyn Error>>
where
    B: DrawingBackend + 'static,
{
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;
    let [x_min, x_max, y_min, y_max] = curves_rect(curves);
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
        let color = Palette99::pick(i);
        chart
            .draw_series(LineSeries::new(curve.iter().map(|&[x, y]| (x, y)), &color))?
            .label(label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
    }
    chart
        .configure_series_labels()
        .background_style(&WHITE)
        .border_style(&BLACK)
        .label_font(font())
        .draw()?;
    Ok(())
}

fn curves_rect(curves: &[(&str, &[[f64; 2]])]) -> [f64; 4] {
    let mut v = [f64::INFINITY, -f64::INFINITY, f64::INFINITY, -f64::INFINITY];
    for [x, y] in curves.iter().flat_map(|(_, curve)| curve.iter().cloned()) {
        if x < v[0] {
            v[0] = x;
        }
        if x > v[1] {
            v[1] = x;
        }
        if y < v[2] {
            v[2] = y;
        }
        if y > v[3] {
            v[3] = y;
        }
    }
    let dx = (v[1] - v[0]).abs();
    let dy = (v[3] - v[2]).abs();
    if dx > dy {
        let cen = (v[2] + v[3]) * 0.5;
        let r = dx * 0.5;
        v[2] = cen - r;
        v[3] = cen + r;
    } else {
        let cen = (v[0] + v[1]) * 0.5;
        let r = dy * 0.5;
        v[0] = cen - r;
        v[1] = cen + r;
    }
    v
}
