use glium::*;
use imgui::*;

use nalgebra::{Perspective3, Unit};
use nalgebra::{Point3, Vector3, Vector4, Matrix4};

use crate::system;
use crate::linalg;
use crate::cube;
use crate::camera;
use crate::messages::{DisplayGoal, Label};
use crate::vertex::{Vertex};

pub struct Scene {
    camera: camera::Camera,

    cube: cube::Cube,

    program: glium::Program,

    dims: Vec<String>,
    labels: Vec<Label>,
    context: String,

    show_debug: bool,
    show_context: bool,
    highlight_color: [f32; 4],
    dark_mode: bool
}

fn to_window_coords(mvp: Matrix4<f32>, width: f32, height: f32, v : Vector3<f32>) -> [f32; 2] {
    let pos = mvp * Vector4::new(v[0], v[1], v[2], 1.0);
    let x_ndc = pos[0] / pos[3];
    let y_ndc = pos[1] / pos[3];

    [
        ((1.0 + x_ndc) / 2.0) * width,
        ((1.0 - y_ndc) / 2.0) * height,
    ]
}

fn face_geometry(face: &cube::Face, color: [f32; 4]) -> Vec<Vertex> {
    vec![
        Vertex::new(face.points[0], color),
        Vertex::new(face.points[1], color),
        Vertex::new(face.points[2], color),
        Vertex::new(face.points[3], color),
        Vertex::new(face.points[0], color),
        Vertex::new(face.points[2], color),
        Vertex::new(face.points[1], color),
        Vertex::new(face.points[3], color),
    ]
}

fn hypercube_geometry(cube: &cube::Cube) -> Vec<Vertex> {
    // FIXME: This is really not efficient!
    // We should probably use a mode that isn't GL_LINES here?
    cube.faces.iter().flat_map(|v| face_geometry(v, [0.0, 0.0, 0.0, 1.0])).collect()
}


fn render_label(ui: &Ui, mvp: Matrix4<f32>, lbl: &Label) {
    let [width, height] = ui.io().display_size;
    let projected = linalg::project(&lbl.position);
    let window_pos = to_window_coords(mvp, width, height, projected);

    let title =
        if lbl.txt.len() > 10 {
            format!("{}...##{}\0", lbl.txt[0..9 - 3].to_owned(), lbl.txt)
        } else {
            format!("{}##{}\0", lbl.txt, lbl.txt)
        };



    let title_imstr = unsafe { ImStr::from_utf8_with_nul_unchecked(title.as_bytes()) };
    // let lbl_imstr = unsafe { ImStr::from_utf8_with_nul_unchecked(lbl.as_bytes()) };
    Window::new(title_imstr)
        .position(window_pos, Condition::Always)
        .size([100.0, 100.0], Condition::Appearing)
        .collapsed(true, Condition::Appearing)
        .build(ui, || {
            ui.text(format!("{}\0", lbl.txt));
        });
}

fn init_scene(display: &glium::Display, DisplayGoal { dims, labels, context }: DisplayGoal) -> Scene {
    let camera = camera::Camera::new();

    let program = program!(display, 140 => {
        vertex: include_str!("../resources/shader.vert"),
        fragment: include_str!("../resources/shader.frag")
    }).unwrap();

    let cube = cube::Cube::new(display, &dims, 1.0);

    Scene {
        camera,
        program,
        cube,
        dims,
        labels,
        context,
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
        render_label(ui, mvp, lbl);
    }

    let [width, height] = ui.io().display_size;
    let [mouse_x, mouse_y] = ui.io().mouse_pos;

    // FIXME: Factor this out
    let mouse_ndc_point = Point3::new(-1.0 + 2.0 * (mouse_x / width), 1.0 - 2.0 * (mouse_y / height),  1.0);
    let mouse_view_point = view.inverse() * projection.unproject_point(&mouse_ndc_point);
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

pub fn display_goal(msg : DisplayGoal) {
    let system = system::init(file!());

    let mut scene = init_scene(&system.display, msg);
    system.main_loop(move |_, display, target, ui| {
        handle_input(ui, &mut scene);
        render_frame(ui, &mut scene, target);
    })
}

pub fn display_hypercube(dims : Vec<String>) {
    let system = system::init(file!());

    let ctx = "render test\0";
    let mut scene = init_scene(&system.display, DisplayGoal { dims, labels: vec![], context: ctx.to_string() });
    system.main_loop(move |_, display, target, ui| {
        handle_input(ui, &mut scene);
        render_frame(ui, &mut scene, target);
    })
}
