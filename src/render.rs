use glium::*;
use imgui::*;

use nalgebra::{Perspective3, Unit};

use crate::{linalg, system};
use crate::cube;
use crate::camera;
use crate::label;
use crate::messages;

pub struct Scene {
    camera: camera::Camera,

    cube: cube::Cube,

    program: glium::Program,

    dims: Vec<String>,
    labels: Vec<label::Label>,
    context: String,

    show_debug: bool,
    show_context: bool,
    highlight_color: [f32; 4],
    dark_mode: bool
}

fn init_scene(display: &glium::Display, msg: &messages::DisplayGoal) -> Scene {
    let camera = camera::Camera::new();

    let program = program!(display, 140 => {
        vertex: include_str!("../resources/shader.vert"),
        fragment: include_str!("../resources/shader.frag")
    }).unwrap();

    let labels = msg.labels.iter().map(|lbl| label::Label::new(&msg.dims, lbl)).collect();
    let cube = cube::Cube::new(display, &msg.dims, 1.0);

    Scene {
        camera,
        program,
        cube,
        labels,
        dims: msg.dims.clone(),
        context: msg.context.clone(),
        show_debug: false,
        show_context: true,
        highlight_color: [1.0, 0.0, 0.0, 1.0],
        dark_mode: false
    }
}

fn render_frame(ui: &Ui, scene : &mut Scene, target: &mut Frame) {
    let [width, height] = ui.io().display_size;

    let eye = scene.camera.eye();
    let view = scene.camera.view();

    let aspect = width / height;
    let fov = 45.0_f32.to_radians();
    let projection = Perspective3::new(aspect, fov, 0.1, 100.0);

    let view_proj = projection.to_homogeneous() * view.to_homogeneous();
    let mvp = view_proj * scene.cube.model.to_homogeneous();

    scene.cube.render(view_proj, &scene.program, target);

    for lbl in &scene.labels {
        lbl.render(mvp, ui);
    }

    let mouse_view_point = view.inverse() * linalg::world_coords(projection, ui.io().display_size, ui.io().mouse_pos);
    let direction = Unit::new_normalize(eye - mouse_view_point);

    let isects = scene.cube.intersections(eye, *direction);
    if let Some((_, face)) = isects.first() {
        scene.cube.render_face(face, view_proj, &scene.program, target);
        ui.tooltip(|| {
            let mut s = String::new();
            for (nm, d) in &face.dims {
                s.push_str(&format!("{} = {}\n", nm, if *d { 1 } else { 0 }));
            }
            ui.text(s);
        });
    };

    // FIXME: Use the begin_ family of functions here to
    // avoid issues with closures.

    // We don't want to have to mutably borrow these fields,
    // as that would prevent us from having any other references
    // to parts of scene, so we just make copies here and then update the
    // scene at the end of the frame.
    let mut show_debug = scene.show_debug;
    let mut show_context = scene.show_context;
    let mut highlight_color = scene.highlight_color;
    let mut dark_mode = scene.dark_mode;

    ui.main_menu_bar(|| {
        ui.menu(im_str!("Menu"), || {
            ui.checkbox(im_str!("Show Context"), &mut show_context);
            ui.checkbox(im_str!("Debug Panel"), &mut show_debug);
        });
    });

    if scene.show_debug {
        Window::new(im_str!("Debug")).build(ui, || {
            ui.text(format!("Camera Position: {} {} {}", eye[0], eye[1], eye[2]));
            if CollapsingHeader::new(im_str!("Intersections")).default_open(false).build(ui) {
                for (isect, _) in isects {
                    ui.text(format!("{} {} {}", isect[0], isect[1], isect[2]))
                }
            }
            ui.spacing();
            ColorPicker::new(im_str!("Highlight Color"), &mut highlight_color).build(ui);
            ui.checkbox(im_str!("Dark Mode"), &mut dark_mode);
        });
    }

    if scene.show_context {
        let ctx = unsafe { ImStr::from_utf8_with_nul_unchecked(scene.context.as_bytes()) };
        Window::new(im_str!("Context")).build(ui, || {
            ui.text_wrapped(ctx)
        });
    }

    scene.show_context = show_context;
    scene.show_debug = show_debug;
    scene.dark_mode = dark_mode;
    scene.highlight_color = highlight_color;
}

fn handle_input(ui: &Ui, scene: &mut Scene) {
    let io = ui.io();
    if !io.want_capture_mouse {
        let [delta_x, delta_y] = io.mouse_delta;
        if ui.is_mouse_down(MouseButton::Left) {
            scene.camera.rotate_azimuth(delta_x / 300.0);
            scene.camera.rotate_polar(delta_y / 300.0);
        }
        scene.camera.zoom(0.1_f32 * io.mouse_wheel);
    }
}

fn handle_message(msg: messages::Message, display: &Display, scene: &mut Scene) {
    match msg {
        messages::Message::DisplayGoal(goal) =>
            *scene = init_scene(display, &goal)
    }
}

pub fn render() {
    let system = system::init(3001, file!());
    let dims = vec!["i".to_string(), "j".to_string(), "k".to_string(), "l".to_string()];

    let ctx = "DEMO\0";
    let scene = init_scene(&system.display, &messages::DisplayGoal { dims, labels: vec![], context: ctx.to_string() });
    system.main_loop(scene, handle_message, |_, display, scene, target, ui| {
        handle_input(ui, scene);
        render_frame(ui, scene, target);
    })
}
