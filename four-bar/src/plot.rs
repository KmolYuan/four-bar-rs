//! The functions used to plot the curve and synthesis result.

pub use plotters::{prelude::*, *};

/// Get font setting.
pub fn font() -> TextStyle<'static> {
    ("Times New Roman", 24).into_font().color(&BLACK)
}

/// Plot the synthesis history.
pub fn plot_history<B>(backend: B, history: &[f64], fitness: f64) -> anyhow::Result<()>
where
    B: DrawingBackend,
    B::ErrorType: 'static,
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
pub fn plot_curve<B>(backend: B, title: &str, curves: &[(&str, &[[f64; 2]])]) -> anyhow::Result<()>
where
    B: DrawingBackend,
    B::ErrorType: 'static,
{
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;
    let [x_min, x_max, y_min, y_max] = bounding_box(curves);
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

/// Get the bounding box of the data, ignore the labels.
pub fn bounding_box<L>(curves: &[(L, &[[f64; 2]])]) -> [f64; 4] {
    let [mut x_min, mut x_max] = [&f64::INFINITY, &-f64::INFINITY];
    let [mut y_min, mut y_max] = [&f64::INFINITY, &-f64::INFINITY];
    for [x, y] in curves.iter().flat_map(|(_, curve)| curve.iter()) {
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
