//! The functions used to plot the curve and synthesis result.

use plotly::{
    common::{Font, Marker, Mode, Title},
    layout::{Axis, Legend, RangeMode, TicksDirection},
    ImageFormat, Layout, Plot, Scatter, Trace,
};

fn font() -> Font {
    const FONT: &str = if cfg!(windows) {
        "Times New Roman"
    } else {
        "Nimbus Roman No9 L"
    };
    Font::new().family(FONT)
}

/// Plot the synthesis history. (SVG)
pub fn plot_history<P>(history: &[f64], fitness: f64, path: P)
where
    P: AsRef<std::path::Path>,
{
    let trace = Scatter::new(0..history.len(), history.iter().cloned()).mode(Mode::LinesMarkers);
    let cap = format!("Convergence Plot (Best Fitness: {:.04})", fitness);
    let axis = |label| {
        Axis::new()
            .mirror(true)
            .show_tick_labels(true)
            .show_line(true)
            .ticks(TicksDirection::Outside)
            .range_mode(RangeMode::NonNegative)
            .title(Title::new(label).font(font().size(18)))
            .tick_font(font().size(18))
    };
    let layout = Layout::new()
        .title(Title::new(&cap).font(font().size(20)))
        .x_axis(axis("Generation"))
        .y_axis(axis("Fitness"));
    let mut plot = Plot::new();
    plot.add_trace(trace);
    plot.set_layout(layout);
    plot.save(path, ImageFormat::SVG, 800, 600, 1.);
}

/// Plot 2D curve. (SVG)
pub fn plot_curve<P>(title: &str, curves: &[(&str, &[[f64; 2]])], path: P)
where
    P: AsRef<std::path::Path>,
{
    let traces = curves
        .iter()
        .map(|&(name, curve)| {
            let x = curve.iter().map(|[x, _]| *x);
            let y = curve.iter().map(|[_, y]| *y);
            Scatter::new(x, y)
                .name(name)
                .marker(Marker::new().max_displayed(30))
                .mode(Mode::LinesMarkers) as Box<dyn Trace>
        })
        .collect();
    let axis = || {
        Axis::new()
            .mirror(true)
            .zero_line(false)
            .auto_range(true)
            .show_line(true)
            .ticks(TicksDirection::Outside)
            .tick_font(font().size(18))
    };
    let layout = Layout::new()
        .title(Title::new(title).font(font().size(20)))
        .legend(Legend::new().font(font().size(18)))
        .x_axis(axis())
        .y_axis(axis());
    let mut plot = Plot::new();
    plot.add_traces(traces);
    plot.set_layout(layout);
    plot.save(path, ImageFormat::SVG, 800, 800, 1.);
}
