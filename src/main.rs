use imgui::{MouseButton};

mod linalg;
mod cube;
mod render;
mod system;


fn main() {
    let system = system::init(file!());
    let labels = vec![
        render::Label { position: vec![1.0, 1.0, 1.0], txt: "Hello".to_string() },
        render::Label { position: vec![1.0, 1.0, -1.0], txt: "There".to_string() },
    ];
    let mut scene = render::init_scene(&system.display, 3, labels);
    system.main_loop(move |_, target, ui| {
        let io = ui.io();
        if !io.want_capture_mouse {
            let [delta_x, delta_y] = io.mouse_delta;
            if ui.is_mouse_down(MouseButton::Left) {
                scene.azimuth += delta_x / 300.0;
                scene.polar += delta_y / 300.0;
            }
            scene.radius += 0.1_f32 * io.mouse_wheel;
        }

        render::render_frame(ui, &scene, target);
    })
}
