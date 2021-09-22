use nalgebra::*;

fn right_pad_vec<T>(x: &mut Vec<T>, len: usize, pad: T)
where
    T: Copy,
{
    while x.len() < len {
        x.push(pad);
    }
}

pub fn project(v : &[f32]) -> Vector3<f32> {
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

    Vector3::new(tmp[0], tmp[1], tmp[2])
}

pub fn window_coords(mvp: Matrix4<f32>, screen_dims: [f32; 2], v : Vector3<f32>) -> [f32; 2] {
    let pos = mvp * Vector4::new(v[0], v[1], v[2], 1.0);
    let x_ndc = pos[0] / pos[3];
    let y_ndc = pos[1] / pos[3];

    [
        ((1.0 + x_ndc) / 2.0) * screen_dims[0],
        ((1.0 - y_ndc) / 2.0) * screen_dims[1],
    ]
}

pub fn world_coords(proj: Perspective3<f32>, screen_dims:[f32; 2], pos: [f32; 2]) -> Point3<f32> {
    let ndc_point = Point3::new(-1.0 + 2.0 * (pos[0] / screen_dims[0]), 1.0 - 2.0 * (pos[1] / screen_dims[1]),  1.0);
    proj.unproject_point(&ndc_point)
}
