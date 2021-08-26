use std::convert::TryInto;

use crate::linalg;

fn insert_bit(bits : u32, ix : u32) -> u32 {
    let upper_mask = u32::MAX << (ix + 1);
    let upper = upper_mask & (bits << 1);
    let lower_mask = (1 << ix) - 1;
    let lower = lower_mask & bits;
    upper | lower
}

fn hypercube_vertices(dim: u32) -> u32 {
    2_u32.pow(dim) * dim
}


pub fn hypercube(dim: u32, size: f32) -> Vec<[f32; 3]> {
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
            points.push(linalg::project(&e0));
            points.push(linalg::project(&e1));
        }
    }
    points
}
