//! The functions used to plot the curve and synthesis result.

use metaheuristics_nature::Report;
use plotters::prelude::*;
use std::path::Path;

const FONT: &str = if cfg!(windows) {
    "Times New Roman"
} else {
    "Nimbus Roman No9 L"
};

/// Plot the synthesis history.
pub fn plot_history<P>(history: &[Report], path: P)
where
    P: AsRef<Path>,
{
    let root = SVGBackend::new(&path, (1600, 900)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let gen_max = history.iter().map(|r| r.gen).max().unwrap();
    let (max_best, _) = find_extreme(history.iter().map(|r| r.best_f));
    let (max_average, _) = find_extreme(history.iter().map(|r| r.average));
    let mut chart = ChartBuilder::on(&root)
        .caption("History", (FONT, 50))
        .x_label_area_size(45)
        .y_label_area_size(50)
        .right_y_label_area_size(50)
        .margin(20)
        .build_cartesian_2d(0..gen_max, 0f64..max_best)
        .unwrap()
        .set_secondary_coord(0..gen_max, 0f64..max_average);
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
    chart
        .configure_secondary_axes()
        .label_style((FONT, 20))
        .axis_desc_style((FONT, 20))
        .y_desc("Average Fitness")
        .draw()
        .unwrap();
    const BLUE: RGBColor = RGBColor(118, 182, 222);
    chart
        .draw_series(LineSeries::new(
            history.iter().map(|r| (r.gen, r.best_f)),
            BLUE.stroke_width(5),
        ))
        .unwrap()
        .label("Best Fitness")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));
    const GREEN: RGBColor = RGBColor(187, 222, 13);
    chart
        .draw_secondary_series(LineSeries::new(
            history.iter().map(|r| (r.gen, r.average)),
            GREEN.stroke_width(5),
        ))
        .unwrap()
        .label("Average Fitness")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));
    chart
        .configure_series_labels()
        .label_font((FONT, 30))
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

/// Plot 2D curve.
pub fn plot_curve<'a, P>(title: &str, curves: &[(&str, &[[f64; 2]], (u8, u8, u8))], path: P)
where
    P: AsRef<Path>,
{
    let iter = curves
        .iter()
        .flat_map(|(_, c, _)| c.iter().flat_map(|c| *c));
    let (p_max, p_min) = find_extreme(iter);
    let root = SVGBackend::new(&path, (1000, 1000)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(title, (FONT, 40))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .margin(20)
        .build_cartesian_2d(p_min..p_max, p_min..p_max)
        .unwrap();
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style((FONT, 20))
        .axis_desc_style((FONT, 20))
        .draw()
        .unwrap();
    for (i, &(name, curve, (r, g, b))) in curves.iter().enumerate() {
        let color = RGBColor(r, g, b);
        chart
            .draw_series(LineSeries::new(
                curve.iter().map(|&[x, y]| (x, y)),
                color.stroke_width(2),
            ))
            .unwrap()
            .label(name)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });
        chart
            .draw_series(curve.iter().step_by(6).map(|&[x, y]| {
                if i % 2 == 1 {
                    Circle::new((x, y), 5, color.stroke_width(1)).into_dyn()
                } else {
                    TriangleMarker::new((x, y), 7, color.stroke_width(1)).into_dyn()
                }
            }))
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
