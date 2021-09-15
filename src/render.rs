use glium::*;
use imgui::*;

use nalgebra::{Isometry3, Perspective3, Unit};
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

// FIXME: There is probably a smarter way of doing this?
impl Vertex {
    fn from_vector(v : Vector3<f32>, color: Vector3<f32>) -> Vertex {
        Vertex { position: [v[0], v[1], v[2]], color: [color[0], color[1], color[2]] }
    }
}

implement_vertex!(Vertex, position, color);

pub struct Scene {
    // Camera Controls
    pub azimuth: f32,
    pub polar: f32,
    pub radius: f32,

    cube: cube::Cube,

    // Graphics Data
    program: glium::Program,
    cube_vbo: glium::VertexBuffer<Vertex>,
    face_vbo: glium::VertexBuffer<Vertex>,

    dim: u32,
    labels: Vec<Label>,
    context: String,

    show_debug: bool,
    show_context: bool,
    debug_render: bool,
}

// fn to_window_coords(mvp: Matrix4<f32>, width: f32, height: f32, v : Vector3<f32>) -> [f32; 2] {
//     let pos = mvp * Vector4::new(v[0], v[1], v[2], 1.0);
//     let x_ndc = pos[0] / pos[3];
//     let y_ndc = pos[1] / pos[3];

//     [
//         ((1.0 + x_ndc) / 2.0) * width,
//         ((1.0 - y_ndc) / 2.0) * height,
//     ]
// }

// fn from_window_coords(mvp: Matrix4<f32>, width: f32, height: f32, w: [f32; 2]) -> Vector3<f32> {
//     let transform = ()
// }

fn face_geometry(face: &cube::Face, color: Vector3<f32>) -> Vec<Vertex> {
    vec![
        Vertex::from_vector(face.points[0], color),
        Vertex::from_vector(face.points[1], color),
        Vertex::from_vector(face.points[2], color),
        Vertex::from_vector(face.points[3], color),
        Vertex::from_vector(face.points[0], color),
        Vertex::from_vector(face.points[2], color),
        Vertex::from_vector(face.points[1], color),
        Vertex::from_vector(face.points[3], color),
    ]
}

fn hypercube_geometry(cube: &cube::Cube) -> Vec<Vertex> {
    // FIXME: This is really not efficient!
    // We should probably use a mode that isn't GL_LINES here?
    cube.faces.iter().flat_map(|v| face_geometry(v, Vector3::new(0.0, 0.0, 0.0))).collect()
}


fn render_label(ui: &Ui, mvp: Matrix4<f32>, lbl: &Label) {
    let [width, height] = ui.io().display_size;
    let projected = linalg::project(&lbl.position);
    // let window_pos = to_window_coords(mvp, width, height, projected);

    // let title =
    //     if lbl.txt.len() > 10 {
    //         format!("{}...##{}\0", lbl.txt[0..9 - 3].to_owned(), lbl.txt)
    //     } else {
    //         format!("{}##{}\0", lbl.txt, lbl.txt)
    //     };



    // let title_imstr = unsafe { ImStr::from_utf8_with_nul_unchecked(title.as_bytes()) };
    // // let lbl_imstr = unsafe { ImStr::from_utf8_with_nul_unchecked(lbl.as_bytes()) };
    // Window::new(title_imstr)
    //     .position(window_pos, Condition::Always)
    //     .size([100.0, 100.0], Condition::Appearing)
    //     .collapsed(true, Condition::Appearing)
    //     .build(ui, || {
    //         ui.text(format!("{}\0", lbl.txt));
    //     });
}

fn init_scene(display: &glium::Display, DisplayGoal { dim, labels, context }: DisplayGoal) -> Scene {
    let azimuth = 90.0_f32.to_radians();
    let polar = 0.0;
    let radius = 4.0;


    let program = program!(display, 140 => {
        vertex: include_str!("../resources/shader.vert"),
        fragment: include_str!("../resources/shader.frag")
    }).unwrap();

    let cube = cube::Cube::new(dim, 1.0);
    let cube_geometry = hypercube_geometry(&cube);
    let cube_vbo = glium::VertexBuffer::dynamic(display, &cube_geometry).unwrap();
    let face_vbo = glium::VertexBuffer::empty_dynamic(display, 8).unwrap();

    Scene {
        azimuth,
        polar,
        radius,
        program,
        cube,
        cube_vbo,
        face_vbo,
        dim,
        labels,
        context,
        show_debug: false,
        show_context: true,
        debug_render: false,
    }
}

fn render_frame(ui: &Ui, scene : &mut Scene, display : &Display, target: &mut Frame) {
    let model : Matrix4<f32> = Matrix4::identity();
    let [width, height] = ui.io().display_size;

    let eye : Point3<f32> = Point3::new(
        scene.radius * scene.polar.cos() * scene.azimuth.cos(),
        scene.radius * scene.polar.sin(),
        scene.radius * scene.polar.cos() * scene.azimuth.sin(),
    );
    let origin : Point3<f32> = Point3::new(0.0, 0.0, 0.0);
    let up : Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
    let view = Isometry3::look_at_rh(&eye, &origin, &up);

    let aspect = width / height;
    let fov = 45.0_f32.to_radians();
    let projection = Perspective3::new(aspect, fov, 0.1, 100.0);

    let mvp = projection.to_homogeneous() * view.to_homogeneous() * model;

    let model_unif : [[f32; 4]; 4] = model.into();
    let view_unif : [[f32; 4]; 4] = view.to_homogeneous().into();
    let projection_unif : [[f32; 4]; 4] = projection.to_homogeneous().into();

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

    let [width, height] = ui.io().display_size;
    let [mouse_x, mouse_y] = ui.io().mouse_pos;
    let mouse_ndc_point = Point3::new(-1.0 + 2.0 * (mouse_x / width), 1.0 - 2.0 * (mouse_y / height),  1.0);
    let mouse_view_point = view.inverse() * projection.unproject_point(&mouse_ndc_point);
    let direction = Unit::new_normalize(eye - mouse_view_point);
    let isects = scene.cube.intersections(eye, *direction);
    if let Some(&(_, face)) = isects.first() {
        scene.face_vbo.write(&face_geometry(face, Vector3::new(1.0, 0.0, 0.0)));
        target.draw(&scene.face_vbo, &glium::index::NoIndices(glium::index::PrimitiveType::LinesList), &scene.program, &uniforms, &draw_params).unwrap();

        ui.tooltip(|| {
            ui.text(im_str!("tooltip"));
        });
    };

    // We don't want to have to mutably borrow these fields,
    // as that would prevent us from having any other references
    // to parts of scene, so we just make copies here and then update the
    // scene at the end of the frame.
    let mut show_debug = scene.show_debug;
    let mut show_context = scene.show_context;
    let mut debug_render = scene.debug_render;
    let mut dim = scene.dim;
    let mut new_cube = None;

    ui.main_menu_bar(|| {
        ui.menu(im_str!("Menu"), || {
            ui.checkbox(im_str!("Show Context"), &mut show_context);
            ui.checkbox(im_str!("Debug Panel"), &mut show_debug);
        });
    });

    if scene.show_debug {
        Window::new(im_str!("Debug")).build(ui, || {
            ui.text(format!("Camera Position: {} {} {}", eye[0], eye[1], eye[2]));
            if CollapsingHeader::new(im_str!("Intersections")).default_open(true).build(ui) {
                for (isect, _) in isects {
                    ui.text(format!("{} {} {}", isect[0], isect[1], isect[2]))
                }
            }
            ui.spacing();
            if CollapsingHeader::new(im_str!("Rendering")).default_open(true).build(ui) {
                ui.checkbox(im_str!("Debug Render Mode"), &mut debug_render);
                if debug_render && Slider::new(im_str!("Dimension")).range(2..=5).build(ui, &mut dim) {
                    let cube = cube::Cube::new(dim, 1.0);
                    let cube_geometry = hypercube_geometry(&cube);
                    new_cube = Some((cube, glium::VertexBuffer::dynamic(display, &cube_geometry).unwrap()));
                }
            }
            ui.spacing();
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
    scene.debug_render = debug_render;
    scene.dim = dim;
    if let Some((cube, cube_vbo)) = new_cube {
        scene.cube = cube;
        scene.cube_vbo = cube_vbo;
    }
}

fn handle_input(ui: &Ui, scene: &mut Scene) {
    let io = ui.io();
    if !io.want_capture_mouse {
        let [delta_x, delta_y] = io.mouse_delta;
        if ui.is_mouse_down(MouseButton::Left) {
            scene.azimuth += delta_x / 300.0;
            scene.polar += delta_y / 300.0;
        }
        scene.radius += 0.1_f32 * io.mouse_wheel;
    }

}

pub fn display_goal(msg : DisplayGoal) {
    let system = system::init(file!());

    let mut scene = init_scene(&system.display, msg);
    system.main_loop(move |_, display, target, ui| {
        handle_input(ui, &mut scene);
        render_frame(ui, &mut scene, display, target);
    })
}

pub fn display_hypercube(dim : u32) {
    let system = system::init(file!());

    let ctx = "render test\0";
    let mut scene = init_scene(&system.display, DisplayGoal { dim, labels: vec![], context: ctx.to_string() });
    system.main_loop(move |_, display, target, ui| {
        handle_input(ui, &mut scene);
        render_frame(ui, &mut scene, display, target);
    })
}
