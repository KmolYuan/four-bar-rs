use eframe::egui::*;
use std::{f64::consts::TAU, path::PathBuf};

pub(crate) fn toggle_btn(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    ui.group(|ui| {
        let res = ui.selectable_label(*on, label);
        if res.clicked() {
            *on = !*on;
        }
        res
    })
    .inner
}

pub(crate) fn small_btn(ui: &mut Ui, icon: &str, tip: &str) -> bool {
    ui.small_button(icon).on_hover_text(tip).clicked()
}

pub(crate) fn url_btn(ui: &mut Ui, icon: &str, tip: &str, url: &str) {
    if small_btn(ui, icon, tip) {
        ui.ctx().open_url(OpenUrl::new_tab(url));
    }
}

pub(crate) fn hint(ui: &mut Ui, tip: &str) -> Response {
    let res = ui
        .add(Label::new(RichText::new("(?)").underline()).selectable(false))
        .on_hover_text(tip);
    if res.enabled() && res.long_touched() {
        res.show_tooltip_ui(|ui| _ = ui.label(tip));
    }
    res
}

pub(crate) fn counter(
    ui: &mut Ui,
    val: &mut usize,
    rng: std::ops::RangeInclusive<usize>,
) -> Response {
    let at_min = val != rng.start();
    let res1 = ui.add_enabled(at_min, Button::new("-"));
    if res1.clicked() {
        *val -= 1;
    }
    ui.label(val.to_string());
    let at_max = val != rng.end();
    let res2 = ui.add_enabled(at_max, Button::new("+"));
    if res2.clicked() {
        *val += 1;
    }
    res1 | res2
}

pub(crate) fn any_i<V>(ui: &mut Ui, val: &mut V) -> Response
where
    V: emath::Numeric,
{
    ui.add(DragValue::new(val))
}

pub(crate) fn unit<V>(ui: &mut Ui, label: &str, val: &mut V, int: f64) -> Response
where
    V: emath::Numeric,
{
    ui.add(DragValue::new(val).prefix(label).speed(int).min_decimals(2))
}

pub(crate) fn nonzero_i<V>(ui: &mut Ui, label: &str, val: &mut V, int: u32) -> Response
where
    V: emath::Numeric,
{
    let dv = DragValue::new(val)
        .prefix(label)
        .speed(int)
        .range(1..=usize::MAX);
    ui.add(dv)
}

pub(crate) fn nonzero_f<V>(ui: &mut Ui, label: &str, val: &mut V, int: f64) -> Response
where
    V: emath::Numeric,
{
    let dv = DragValue::new(val)
        .prefix(label)
        .speed(int)
        .range(1e-2..=f64::MAX)
        .min_decimals(2);
    ui.add(dv)
}

pub(crate) fn angle(ui: &mut Ui, label: &str, val: &mut f64, suffix: &str) -> Response {
    ui.horizontal(|ui| {
        if suffix.is_empty() {
            *val = val.rem_euclid(TAU);
        }
        let mut deg = val.to_degrees();
        let dv = DragValue::new(&mut deg)
            .prefix(label)
            .suffix(format!(" deg{suffix}"))
            .min_decimals(2)
            .speed(1.);
        let res = ui.add(dv);
        if res.changed() {
            *val = deg.to_radians();
        }
        let dv = DragValue::new(val)
            .suffix(format!(" rad{suffix}"))
            .min_decimals(2)
            .speed(0.01);
        res | ui.add(dv)
    })
    .inner
}

pub(crate) fn percent(ui: &mut Ui, label: &str, val: &mut f64) -> Response {
    let dv = DragValue::new(val)
        .prefix(label)
        .custom_formatter(|v, _| format!("{:.04}%", v * 100.))
        .speed(0.1)
        .range(1e-2..=f64::MAX)
        .min_decimals(2);
    ui.add(dv)
}

pub(crate) fn check_on<V, F>(ui: &mut Ui, label: &str, val: &mut Option<V>, f: F) -> Response
where
    V: Default,
    F: FnOnce(&mut Ui, &mut V) -> Response,
{
    ui.horizontal(|ui| {
        let mut enable = val.is_some();
        let mut ret = ui.checkbox(&mut enable, label);
        if !enable {
            *val = None;
        } else if val.is_none() {
            *val = Some(V::default());
        }
        if let Some(val) = val {
            ret |= f(ui, val);
        }
        ret
    })
    .inner
}

pub(crate) fn path_label(ui: &mut Ui, icon: &str, path: Option<&PathBuf>, warn: &str) -> Response {
    ui.horizontal(|ui| {
        ui.label(icon);
        if let Some(path) = path {
            let path_str = path.to_string_lossy();
            if path.as_os_str().len() < 30 {
                ui.label(path_str)
            } else {
                let path = std::path::Path::new("...").join(path.file_name().unwrap());
                ui.label(path.to_string_lossy()).on_hover_text(path_str)
            }
        } else {
            ui.colored_label(Color32::RED, warn)
        }
    })
    .inner
}

pub(crate) fn table<const N: usize>(ui: &mut Ui, xs: &mut Vec<[f64; N]>) {
    fn render<const N: usize>(ui: &mut Ui, c: &mut [f64; N]) -> bool {
        let keep = !ui.button("✖").clicked();
        let labels: &'static [&'static str] = match N {
            2 => &["x: ", "y: "],
            3 => &["x: ", "y: ", "z: "],
            4 => &["x: ", "y: ", "a: ", "b: "],
            6 => &["x: ", "y: ", "z: ", "a: ", "b: ", "c: "],
            _ => unreachable!(),
        };
        for (c, label) in c.iter_mut().zip(labels) {
            let w = DragValue::new(c)
                .prefix(label)
                .speed(0.01)
                .fixed_decimals(4);
            ui.add(w);
        }
        keep
    }

    let rows = xs.len();
    if rows == 0 {
        ui.group(|ui| ui.label("No curve"));
        return;
    }
    if ui.button("✖ Clear All").clicked() {
        xs.clear();
    }
    let space = ui.spacing().interact_size.y;
    ui.group(|ui| {
        ScrollArea::vertical()
            .max_height(150.)
            .auto_shrink([false; 2])
            .show_rows(ui, space, rows, |ui, rng| {
                let mut i = 0;
                xs.retain_mut(|c| {
                    let hidden = !rng.contains(&i);
                    i += 1;
                    hidden || ui.horizontal(|ui| render(ui, c)).inner
                });
            });
    });
}

pub(crate) fn combo_enum<H, E, F, T, const N: usize>(
    ui: &mut Ui,
    id: H,
    value: &mut E,
    list: [E; N],
    name: F,
) where
    H: std::hash::Hash,
    E: PartialEq + Clone,
    F: Fn(&E) -> T,
    T: Into<WidgetText>,
{
    let mut i = list.iter().position(|opt| opt == value).unwrap();
    if ComboBox::from_id_salt(id)
        .selected_text(name(value))
        .show_index(ui, &mut i, N, |i| name(&list[i]))
        .changed()
    {
        *value = list[i].clone();
    }
}

#[inline]
pub(crate) fn static_plot(name: &str) -> egui_plot::Plot {
    egui_plot::Plot::new(name)
        .legend(Default::default())
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
}
