//! The functions used to plot the curve and synthesis result.

use plotters::prelude::*;
use std::error::Error;

fn font() -> TextStyle<'static> {
    ("Times New Roman", 20).into_font().color(&BLACK)
}

/// Plot the synthesis history.
pub fn plot_history<B>(backend: B, history: &[f64], fitness: f64) -> Result<(), Box<dyn Error>>
where
    B: DrawingBackend + 'static,
{
    let root = backend.into_drawing_area();
    root.fill(&TRANSPARENT)?;
    let cap = format!("Convergence Plot (Best Fitness: {:.04})", fitness);
    let max_fitness = history
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(cap, font())
        .set_label_area_size(LabelAreaPosition::Left, (8).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
        .margin((1).percent())
        .build_cartesian_2d(0..history.len(), 0.0..*max_fitness)?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_desc("Generation")
        .x_label_style(font())
        .x_labels(30)
        .y_desc("Fitness")
        .y_label_style(font())
        .y_labels(30)
        .draw()?;
    chart.draw_series(LineSeries::new(
        history.iter().enumerate().map(|(i, x)| (i, *x)),
        &BLUE,
    ))?;
    chart
        .configure_series_labels()
        .background_style(&TRANSPARENT)
        .border_style(&BLACK)
        .draw()?;
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
    root.fill(&TRANSPARENT)?;
    let mut x_min = f64::INFINITY;
    let mut x_max = -f64::INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = -f64::INFINITY;
    for [x, y] in curves.iter().flat_map(|(_, curve)| curve.iter().cloned()) {
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
    let mut chart = ChartBuilder::on(&root)
        .caption(title, font())
        .set_label_area_size(LabelAreaPosition::Left, (8).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
        .margin((1).percent())
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    chart.configure_mesh().draw()?;
    for &(label, curve) in curves {
        chart
            .draw_series(LineSeries::new(curve.iter().map(|&[x, y]| (x, y)), &BLUE))?
            .label(label)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    }
    chart
        .configure_series_labels()
        .background_style(&TRANSPARENT)
        .border_style(&BLACK)
        .draw()?;
    Ok(())
}
