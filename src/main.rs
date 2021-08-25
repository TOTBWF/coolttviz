extern crate glium;

use std::*;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};

use nalgebra::vector;
use nalgebra::{Point3, Vector3, Vector4, Matrix4};
use glium::*;
use glium::glutin::event::{WindowEvent, MouseButton, ElementState, MouseScrollDelta};
use imgui::*;

mod support;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
}

implement_vertex!(Vertex, position);
struct Scene {

    // Camera Controls
    azimuth: f32,
    polar: f32,
    radius: f32,

    // Graphics Data
    program: glium::Program,
    cube_vbo: glium::VertexBuffer<Vertex>
}

fn hypercube_vertices(dim: u32) -> u32 {
    2_u32.pow(dim) * dim
}

fn insert_bit(bits : u32, ix : u32) -> u32 {
    let upper_mask = u32::MAX << (ix + 1);
    let upper = upper_mask & (bits << 1);
    let lower_mask = (1 << ix) - 1;
    let lower = lower_mask & bits;
    upper | lower
}

fn right_pad_vec<T>(x: &mut Vec<T>, len: usize, pad: T)
where
    T: Copy,
{
    while x.len() < len {
        x.push(pad);
    }
}

// FIXME: This could be smarter
fn project(v : &[f32]) -> Vertex {
    let t : f32 = (45.0_f32.to_radians() / 2.0).tan();
    let mut tmp = vec![0.0; v.len()];
    tmp.copy_from_slice(v);
    right_pad_vec(&mut tmp, 3, 0.0);


    for k in (4..=v.len()).rev() {
        let proj = tmp[k - 1] + 3.0;
        for p in &mut tmp {
            *p = (t * *p) / proj;
        }
    }

    Vertex {
        position: [tmp[0], tmp[1], tmp[2]]
    }
}

fn to_ndc(v : Vertex) -> Vector4<f32> {
    Vector4::new(v.position[0], v.position[1], v.position[2], 1.0)
}

fn hypercube(dim: u32, size: f32) -> Vec<Vertex> {
    let capacity = (hypercube_vertices(dim) * dim).try_into().unwrap();
    let mut points = Vec::with_capacity(capacity);
    let dim_size : usize = dim.try_into().unwrap();
    let mut e0 = vec![0.0; dim_size];
    let mut e1 = vec![0.0; dim_size];


    // We start by selecting what dimension we will draw the line along.
    for line_dim in 0..dim {
        // Next, we need to pick values for all the other dimensions.
        for pos in 0..2_u32.pow(dim - 1) {
            let point = insert_bit(pos, line_dim);
            for i in 0..dim {
                if i == line_dim {
                    e0[i as usize] = -size;
                    e1[i as usize] = size
                } else {
                    let c : f32 = if ((1 << i) & point) == 0 {
                        -size
                    } else {
                        size
                    };
                    e0[i as usize] = c;
                    e1[i as usize] = c;
                }
            }
            points.push(project(&e0));
            points.push(project(&e1));
        }
    }
    points
}

fn init_scene(display: &glium::Display, dim : u32) -> Scene {
    let azimuth = 90.0_f32.to_radians();
    let polar = 0.0;
    let radius = 4.0;

    let program = program!(display, 140 => {
        vertex: include_str!("../resources/shader.vert"),
        fragment: include_str!("../resources/shader.frag")
    }).unwrap();

    let cube_vbo = hypercube(dim, 1.0);

    let cube_vbo = glium::VertexBuffer::dynamic(display, &cube_vbo).unwrap();

    Scene {
        azimuth,
        polar,
        radius,

        program,
        cube_vbo
    }
}

fn render_label(mvp: Matrix4<f32>, ui: &Ui, txt: &str, pos: Vec<f32>) {
    let label_pos = mvp * to_ndc(project(&pos));

    let x_ndc = label_pos[0] / label_pos[3];
    let y_ndc = label_pos[1] / label_pos[3];

    let window_pos = [
        ((1.0 + x_ndc) / 2.0) * 1024.0,
        ((1.0 - y_ndc) / 2.0) * 768.0,
    ];

    // We need to set the 'w' component to 1 here to make the conversion
    // into Normalized Device Coordinates work.
    let title = unsafe { ImStr::from_utf8_with_nul_unchecked(txt.as_bytes()) };
    Window::new(title)
        .position(window_pos, Condition::Always)
        .collapsed(true, Condition::Appearing)
        .build(ui, || {})
}

fn render_scene(ui: &Ui, scene : &Scene, target: &mut Frame) {
    let model : Matrix4<f32> = Matrix4::identity();

    let eye : Point3<f32> = Point3::new(
        scene.radius * scene.polar.cos() * scene.azimuth.cos(),
        scene.radius * scene.polar.sin(),
        scene.radius * scene.polar.cos() * scene.azimuth.sin(),
    );
    let origin : Point3<f32> = Point3::new(0.0, 0.0, 0.0);
    let up : Vector3<f32> = vector![0.0, 1.0, 0.0];
    let view = Matrix4::look_at_rh(&eye, &origin, &up);

    // FIXME: Use real values here!
    let aspect = 800.0/600.0;
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

    target.draw(&scene.cube_vbo, &glium::index::NoIndices(glium::index::PrimitiveType::LinesList), &scene.program, &uniforms, &Default::default()).unwrap();
    render_label(mvp, ui, "Hello", vec![1.0, 1.0, 1.0, 0.0])
}

fn main() {
    let system = support::init(file!());
    let scene = Arc::new(Mutex::new(init_scene(&system.display, 4)));
    // FIXME: It's probably easier to roll the display + input handling code together into the same function.
    let display_scene = Arc::clone(&scene);
    let input_scene = Arc::clone(&scene);
    let mut pressed = false;
    let mut last_x = 0.0;
    let mut last_y = 0.0;
    system.main_loop(
    move |_, target, ui| {
        render_scene(ui, &display_scene.lock().unwrap(), target);
        Window::new(im_str!("Hello world"))
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("こんにちは世界！"));
                ui.text(im_str!("This...is...imgui-rs!"));
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));
            });
        },
        move |event| {
            match event {
                WindowEvent::MouseInput { button: MouseButton::Left, state: ElementState::Released, .. } => pressed = false,
                WindowEvent::MouseInput { button: MouseButton::Left, state: ElementState::Pressed, .. } => pressed = true,
                WindowEvent::CursorMoved { position, .. } => {
                    if pressed {
                        let mut s = input_scene.lock().unwrap();
                        s.azimuth += (position.x as f32 - last_x) / 300.0;
                        s.polar += (position.y as f32 - last_y) / 300.0;
                    }
                    last_x = position.x as f32;
                    last_y = position.y as f32;
                },
                WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, y), ..} => {
                    let mut s = input_scene.lock().unwrap();
                    s.radius += 0.1_f32 * y.signum()
                }
                _ => ()
            }
        });
}
