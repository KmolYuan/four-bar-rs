use crate::as_values::as_values;
use eframe::egui::{
    plot::{Line, PlotUi, Points, Polygon},
    Color32,
};
use four_bar::{FourBar, Mechanism};
use std::sync::{Arc, RwLock};

fn draw_link2(a: [f64; 2], b: [f64; 2]) -> Line {
    Line::new(as_values(&[a, b]))
        .width(3.)
        .color(Color32::from_rgb(165, 151, 132))
}

fn draw_link3(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Polygon {
    Polygon::new(as_values(&[a, b, c]))
        .width(3.)
        .fill_alpha(0.6)
        .color(Color32::from_rgb(165, 151, 132))
}

pub(crate) fn draw_path(name: &str, path: &[[f64; 2]]) -> Line {
    Line::new(as_values(path)).name(name).width(3.)
}

#[derive(Default)]
pub(crate) struct Canvas {
    pub(crate) path1: Vec<[f64; 2]>,
    pub(crate) path2: Vec<[f64; 2]>,
    pub(crate) path3: Vec<[f64; 2]>,
    joints: [[f64; 2]; 5],
}

impl Canvas {
    pub(crate) fn update(&mut self, four_bar: Arc<RwLock<FourBar>>, angle: f64, n: usize) {
        let m = Mechanism::four_bar(&*four_bar.read().unwrap());
        m.apply(angle, [0, 1, 2, 3, 4], &mut self.joints);
        let [path1, path2, path3] = m.four_bar_loop_all(0., n);
        self.path1 = path1;
        self.path2 = path2;
        self.path3 = path3;
    }

    pub(crate) fn ui(&self, ui: &mut PlotUi) {
        ui.line(draw_link2(self.joints[0], self.joints[2]));
        ui.line(draw_link2(self.joints[1], self.joints[3]));
        ui.polygon(draw_link3(self.joints[2], self.joints[3], self.joints[4]));
        ui.points(
            Points::new(as_values(&[self.joints[0], self.joints[1]]))
                .radius(7.)
                .color(Color32::from_rgb(93, 69, 56)),
        );
        ui.points(
            Points::new(as_values(&[self.joints[2], self.joints[3], self.joints[4]]))
                .radius(5.)
                .color(Color32::from_rgb(128, 96, 77)),
        );
        ui.line(draw_path("Crank pivot", &self.path1));
        ui.line(draw_path("Follower pivot", &self.path2));
        ui.line(draw_path("Coupler pivot", &self.path3));
    }
}
