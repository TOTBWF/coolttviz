use imgui::*;

use nalgebra::{Point3, Vector3, Vector4, Matrix4};

use crate::linalg;
use crate::messages;

pub struct Label {
    pub position: Vec<f32>,
    pub txt: String
}

// FIXME: Move this somewhere else!
fn to_window_coords(mvp: Matrix4<f32>, width: f32, height: f32, v : Vector3<f32>) -> [f32; 2] {
    let pos = mvp * Vector4::new(v[0], v[1], v[2], 1.0);
    let x_ndc = pos[0] / pos[3];
    let y_ndc = pos[1] / pos[3];

    [
        ((1.0 + x_ndc) / 2.0) * width,
        ((1.0 - y_ndc) / 2.0) * height,
    ]
}

impl Label {
    pub fn new(dims: &[String], lbl: &messages::Label) -> Label {
        let mut position = Vec::new();
        for dim in dims {
            match lbl.position.get(dim) {
                Some(pos) => position.push(*pos),
                None => position.push(0.0)
            }
        }
        Label {
            position,
            txt: lbl.txt.clone()
        }
    }
}

impl Label {
    pub fn render(&self, mvp: Matrix4<f32>, ui: &Ui) {
        let [width, height] = ui.io().display_size;
        let projected = linalg::project(&self.position);
        let window_pos = to_window_coords(mvp, width, height, projected);

        // We want to truncate the label titles here, as they can get absolutely massive.
        let title =
            if self.txt.len() > 10 {
                format!("{}...##{}\0", self.txt[0..9 - 3].to_owned(), self.txt)
            } else {
                format!("{}##{}\0", self.txt, self.txt)
            };

        let title_imstr = unsafe { ImStr::from_utf8_with_nul_unchecked(title.as_bytes()) };
        Window::new(title_imstr)
            .position(window_pos, Condition::Always)
            .size([100.0, 100.0], Condition::Appearing)
            .collapsed(true, Condition::Appearing)
            .build(ui, || {
                ui.text(format!("{}\0", self.txt));
            });
    }
}
