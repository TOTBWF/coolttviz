use glium::*;
use imgui::*;

use nalgebra::vector;
use nalgebra::{Point3, Vector3, Vector4, Matrix4};

use crate::linalg;
use crate::cube;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
}

implement_vertex!(Vertex, position);

pub struct Label {
    pub position: Vec<f32>,
    pub txt: String
}

pub struct Scene {
    // Camera Controls
    pub azimuth: f32,
    pub polar: f32,
    pub radius: f32,

    // Graphics Data
    program: glium::Program,
    cube_vbo: glium::VertexBuffer<Vertex>,

    labels: Vec<Label>,
}

fn to_window_coords(mvp: Matrix4<f32>, width: f32, height: f32, v : Vertex) -> [f32; 2] {
    let pos = mvp * Vector4::new(v.position[0], v.position[1], v.position[2], 1.0);
    let x_ndc = pos[0] / pos[3];
    let y_ndc = pos[1] / pos[3];

    [
        ((1.0 + x_ndc) / 2.0) * width,
        ((1.0 - y_ndc) / 2.0) * height,
    ]
}


fn render_label(ui: &Ui, mvp: Matrix4<f32>, lbl: &Label) {
    let [width, height] = ui.io().display_size;
    let projected = Vertex { position: linalg::project(&lbl.position) };
    let window_pos = to_window_coords(mvp, width, height, projected);

    // We need to set the 'w' component to 1 here to make the conversion
    // into Normalized Device Coordinates work.
    let title = unsafe { ImStr::from_utf8_with_nul_unchecked(lbl.txt.as_bytes()) };
    Window::new(title)
        .position(window_pos, Condition::Always)
        .size([100.0, 100.0], Condition::Appearing)
        .collapsed(true, Condition::Appearing)
        .build(ui, || {})
}

pub fn init_scene(display: &glium::Display, dim : u32, labels: Vec<Label>) -> Scene {
    let azimuth = 90.0_f32.to_radians();
    let polar = 0.0;
    let radius = 4.0;


    let program = program!(display, 140 => {
        vertex: include_str!("../resources/shader.vert"),
        fragment: include_str!("../resources/shader.frag")
    }).unwrap();

    let cube_geometry = cube::hypercube(dim, 1.0);
    let cube_vbo = glium::VertexBuffer::dynamic(display, &cube_geometry).unwrap();

    Scene {
        azimuth,
        polar,
        radius,
        program,
        cube_vbo,
        labels
    }
}

pub fn render_frame(ui: &Ui, scene : &Scene, target: &mut Frame) {
    let model : Matrix4<f32> = Matrix4::identity();
    let [width, height] = ui.io().display_size;

    let eye : Point3<f32> = Point3::new(
        scene.radius * scene.polar.cos() * scene.azimuth.cos(),
        scene.radius * scene.polar.sin(),
        scene.radius * scene.polar.cos() * scene.azimuth.sin(),
    );
    let origin : Point3<f32> = Point3::new(0.0, 0.0, 0.0);
    let up : Vector3<f32> = vector![0.0, 1.0, 0.0];
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
    for lbl in &scene.labels {
        render_label(ui, mvp, lbl);
    }
}
