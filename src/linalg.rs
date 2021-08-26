fn right_pad_vec<T>(x: &mut Vec<T>, len: usize, pad: T)
where
    T: Copy,
{
    while x.len() < len {
        x.push(pad);
    }
}

pub fn project(v : &[f32]) -> [f32; 3] {
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

    [tmp[0], tmp[1], tmp[2]]
}
