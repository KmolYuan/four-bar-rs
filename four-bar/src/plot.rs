//! The functions used to plot the curve and synthesis result.

use plotters::prelude::*;
use std::path::Path;

const FONT: &str = if cfg!(windows) {
    "Times New Roman"
} else {
    "Nimbus Roman No9 L"
};

/// Plot the synthesis history. (SVG)
pub fn plot_history<P>(history: &[f64], seed: u128, fitness: f64, path: P)
where
    P: AsRef<Path>,
{
    let root = SVGBackend::new(&path, (1600, 900)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let (max_best, _) = find_extreme(history.iter().cloned());
    let caption = format!("History (Best Fitness: {:.04}, Seed: {})", fitness, seed);
    let mut chart = ChartBuilder::on(&root)
        .caption(caption, (FONT, 50))
        .x_label_area_size(45)
        .y_label_area_size(50)
        .right_y_label_area_size(50)
        .margin(20)
        .build_cartesian_2d(0..history.len(), 0f64..max_best)
        .unwrap();
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style((FONT, 20))
        .axis_desc_style((FONT, 20))
        .x_desc("Generation")
        .y_desc("Best Fitness")
        .draw()
        .unwrap();
    const BLUE: RGBColor = RGBColor(118, 182, 222);
    chart
        .draw_series(LineSeries::new(
            history.iter().cloned().enumerate(),
            BLUE.stroke_width(5),
        ))
        .unwrap()
        .label("Best Fitness")
        .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], BLUE));
    chart
        .configure_series_labels()
        .label_font((FONT, 30))
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

/// Plot 2D curve. (SVG)
pub fn plot_curve<P>(title: &str, curves: &[(&str, &[[f64; 2]])], path: P)
where
    P: AsRef<Path>,
{
    let xs = curves.iter().flat_map(|(_, c)| c.iter().map(|&[x, _]| x));
    let ys = curves.iter().flat_map(|(_, c)| c.iter().map(|&[_, y]| y));
    let (x_max, x_min) = find_extreme(xs);
    let (y_max, y_min) = find_extreme(ys);
    let root = SVGBackend::new(&path, (1000, 1000)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(title, (FONT, 40))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .margin(20)
        .build_cartesian_2d((x_min - 4.)..x_max + 4., (y_min - 4.)..y_max + 4.)
        .unwrap();
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style((FONT, 20))
        .axis_desc_style((FONT, 20))
        .draw()
        .unwrap();
    for (i, &(name, curve)) in curves.iter().enumerate() {
        let color = Palette99::pick(i);
        let line = color.stroke_width(2);
        chart
            .draw_series(LineSeries::new(
                curve.iter().map(|&[x, y]| (x, y)),
                line.clone(),
            ))
            .unwrap()
            .label(name)
            .legend(move |(x, y)| PathElement::new([(x, y), (x + 20, y)], line.clone()));
        let step = if curve.len() > 20 {
            curve.len() / 20
        } else {
            1
        };
        chart
            .draw_series(
                curve
                    .iter()
                    .step_by(step)
                    .map(|&[x, y]| TriangleMarker::new((x, y), 7, &color).into_dyn()),
            )
            .unwrap();
    }
    chart
        .configure_series_labels()
        .label_font((FONT, 30))
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

fn find_extreme<I>(mut iter: I) -> (f64, f64)
where
    I: Iterator<Item = f64>,
{
    let mut max = -f64::INFINITY;
    let mut min = f64::INFINITY;
    for v in &mut iter {
        if !v.is_finite() {
            continue;
        }
        if v > max {
            max = v;
        }
        if v < min {
            min = v;
        }
    }
    (max, min)
}
