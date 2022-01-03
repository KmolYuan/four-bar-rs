use crate::as_values::as_values;
use eframe::egui::{
    plot::{Line, PlotUi, Points, Polygon},
    Color32,
};
use four_bar::{FourBar, Mechanism};
use std::sync::{Arc, RwLock};

const JOINT_COLOR: Color32 = Color32::from_rgb(93, 69, 56);
const LINK_COLOR: Color32 = Color32::from_rgb(165, 151, 132);

fn draw_link(ui: &mut PlotUi, points: &[[f64; 2]]) {
    let values = as_values(points);
    if points.len() == 2 {
        ui.line(Line::new(values).width(3.).color(LINK_COLOR));
    } else {
        let polygon = Polygon::new(values)
            .width(3.)
            .fill_alpha(0.6)
            .color(LINK_COLOR);
        ui.polygon(polygon);
    }
}

fn draw_joints(ui: &mut PlotUi, points: &[[f64; 2]]) {
    ui.points(Points::new(as_values(points)).radius(5.).color(JOINT_COLOR));
}

#[derive(Default)]
pub(crate) struct Canvas {
    pub(crate) path: [Vec<[f64; 2]>; 3],
    joints: [[f64; 2]; 5],
}

impl Canvas {
    pub(crate) fn update(&mut self, four_bar: Arc<RwLock<FourBar>>, angle: f64, n: usize) {
        let m = Mechanism::four_bar(&*four_bar.read().unwrap());
        m.apply(angle, [0, 1, 2, 3, 4], &mut self.joints);
        let [path1, path2, path3] = m.four_bar_loop_all(0., n);
        self.path[0] = path1;
        self.path[1] = path2;
        self.path[2] = path3;
    }

    pub(crate) fn ui(&self, ui: &mut PlotUi) {
        draw_link(ui, &[self.joints[0], self.joints[2]]);
        draw_link(ui, &[self.joints[1], self.joints[3]]);
        draw_link(ui, &[self.joints[2], self.joints[3], self.joints[4]]);
        draw_joints(ui, &self.joints);
        let path_names = ["Crank pivot", "Follower pivot", "Coupler pivot"];
        for (path, name) in self.path.iter().zip(path_names) {
            ui.line(Line::new(as_values(path)).name(name).width(3.));
        }
    }
}
