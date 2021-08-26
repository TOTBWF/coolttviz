use glium::*;
use imgui::*;

use nalgebra::{Point3, Vector3, Vector4, Matrix4};

use crate::system;
use crate::linalg;
use crate::cube;
use crate::messages::{DisplayGoal, Label};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3]
}

implement_vertex!(Vertex, position, color);

pub struct Scene {
    // Camera Controls
    pub azimuth: f32,
    pub polar: f32,
    pub radius: f32,

    // Graphics Data
    program: glium::Program,
    cube_vbo: glium::VertexBuffer<Vertex>,

    labels: Vec<Label>,
    context: String
}

fn to_window_coords(mvp: Matrix4<f32>, width: f32, height: f32, v : [f32; 3]) -> [f32; 2] {
    let pos = mvp * Vector4::new(v[0], v[1], v[2], 1.0);
    let x_ndc = pos[0] / pos[3];
    let y_ndc = pos[1] / pos[3];

    [
        ((1.0 + x_ndc) / 2.0) * width,
        ((1.0 - y_ndc) / 2.0) * height,
    ]
}

fn hypercube_geometry(dim: u32, size: f32) -> Vec<Vertex> {
    let vertices = cube::hypercube(dim, size);
    vertices.iter()
        .map(|v| Vertex { position: *v, color: [0.0, 0.0, 0.0] })
        .collect()
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

fn init_scene(display: &glium::Display, DisplayGoal { dim, labels, context }: DisplayGoal) -> Scene {
    let azimuth = 90.0_f32.to_radians();
    let polar = 0.0;
    let radius = 4.0;


    let program = program!(display, 140 => {
        vertex: include_str!("../resources/shader.vert"),
        fragment: include_str!("../resources/shader.frag")
    }).unwrap();

    let cube_geometry = hypercube_geometry(dim, 1.0);
    let cube_vbo = glium::VertexBuffer::dynamic(display, &cube_geometry).unwrap();

    Scene {
        azimuth,
        polar,
        radius,
        program,
        cube_vbo,
        labels,
        context
    }
}

fn render_frame(ui: &Ui, scene : &Scene, target: &mut Frame) {
    let model : Matrix4<f32> = Matrix4::identity();
    let [width, height] = ui.io().display_size;

    let eye : Point3<f32> = Point3::new(
        scene.radius * scene.polar.cos() * scene.azimuth.cos(),
        scene.radius * scene.polar.sin(),
        scene.radius * scene.polar.cos() * scene.azimuth.sin(),
    );
    let origin : Point3<f32> = Point3::new(0.0, 0.0, 0.0);
    let up : Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
    let view = Matrix4::look_at_rh(&eye, &origin, &up);

    let aspect = width / height;
    let fov = 45.0_f32.to_radians();
    let projection = Matrix4::new_perspective(aspect, fov, 0.1, 100.0);

    let mvp = projection * view * model;

    let model_unif : [[f32; 4]; 4] = model.into();
    let view_unif : [[f32; 4]; 4] = view.into();
    let projection_unif : [[f32; 4]; 4] = projection.into();

    let uniforms = uniform! {
        model: model_unif,
        view: view_unif,
        projection: projection_unif
    };

    let draw_params = Default::default();

    target.draw(&scene.cube_vbo, &glium::index::NoIndices(glium::index::PrimitiveType::LinesList), &scene.program, &uniforms, &draw_params).unwrap();

    let ctx = unsafe { ImStr::from_utf8_with_nul_unchecked(scene.context.as_bytes()) };
    Window::new(im_str!("Context")).build(ui, || {
        ui.text_wrapped(ctx)
    });

    for lbl in &scene.labels {
        render_label(ui, mvp, lbl);
    }
}

pub fn display_goal(msg : DisplayGoal) {
    let system = system::init(file!());

    let mut scene = init_scene(&system.display, msg);
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

        render_frame(ui, &scene, target);
    })
}
