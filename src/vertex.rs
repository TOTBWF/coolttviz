use glium::*;
use nalgebra::Vector3;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4]
}


impl Vertex {
    pub fn new(v : Vector3<f32>, color: [f32; 4]) -> Vertex {
        let mut position = [0.0; 3];
        position.copy_from_slice(v.as_slice());
        Vertex { position, color }
    }
}

implement_vertex!(Vertex, position, color);
