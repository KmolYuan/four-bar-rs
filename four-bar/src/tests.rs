use crate::*;
use indicatif::ProgressBar;
use metaheuristics_nature::Report;
use plotters::prelude::*;
use std::{f64::consts::TAU, path::Path};

#[test]
fn planar() {
    let target = Mechanism::four_bar((0., 0.), 0., 90., 35., 70., 70., 77.0875, 5.88785793416)
        .four_bar_loop(TAU / 6., 360);
    let gen = 40;
    let pb = ProgressBar::new(gen as u64);
    let (mut ans, history) = synthesis::synthesis(&target, gen, 200, |r| {
        pb.set_position(r.gen as u64);
        true
    });
    pb.finish();
    let path = ans.four_bar_loop(0., 360);
    plot_curve(
        &[
            ("Target", &target, (221, 51, 85)),
            ("Optimized", &path, (118, 182, 222)),
        ],
        "result.svg",
    );
    plot_history(&history, "history.svg");
}

pub fn plot_curve<'a, S, P>(curves: &[(S, &[[f64; 2]], (u8, u8, u8))], path: P)
where
    S: ToString + Copy,
    P: AsRef<Path>,
{
    let mut p_max = 0.;
    let mut p_min = f64::INFINITY;
    for (_, curve, _) in curves.iter() {
        let max = curve
            .iter()
            .fold(-f64::INFINITY, |v, &[x, y]| v.max(x.max(y)));
        let min = curve
            .iter()
            .fold(f64::INFINITY, |v, &[x, y]| v.min(x.min(y)));
        if max > p_max {
            p_max = max;
        }
        if min < p_min {
            p_min = min;
        }
    }
    let root = SVGBackend::new(&path, (1000, 1000)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption("Curve", ("sans-serif", 50))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .margin(20)
        .build_cartesian_2d(p_min..p_max, p_min..p_max)
        .unwrap();
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
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
            .label(name.to_string())
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
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

pub fn plot_history<P>(history: &[Report], path: P)
where
    P: AsRef<Path>,
{
    let root = SVGBackend::new(&path, (1600, 900)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption("History", ("sans-serif", 50))
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(20)
        .build_cartesian_2d(
            0..history.iter().map(|r| r.gen).max().unwrap(),
            0f64..history.iter().map(|r| r.best_f).fold(0., |a, b| b.max(a)),
        )
        .unwrap();
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_desc("Generation")
        .y_desc("Fitness")
        .draw()
        .unwrap();
    chart
        .draw_series(LineSeries::new(
            history.iter().map(|r| (r.gen, r.best_f)),
            RGBColor(118, 182, 222).stroke_width(5),
        ))
        .unwrap()
        .label("History")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RGBColor(118, 182, 222)));
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}
