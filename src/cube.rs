use nalgebra::{Point3, Vector3};
use ordered_float::NotNan;

use crate::linalg;

// Insert a zero bit at 'ix', shifting over the upper bits to compensate.
fn insert_bit(bits : u32, ix : u32) -> u32 {
    let upper_mask = u32::MAX << (ix + 1);
    let upper = upper_mask & (bits << 1);
    let lower_mask = (1 << ix) - 1;
    let lower = lower_mask & bits;
    upper | lower
}

// Turn the bits of a u32 into a n-dimensional point.
fn point(bits: u32, dim: u32, size: f32) -> Vec<f32> {
    let mut v = vec![0.0; dim as usize];
    for i in 0..dim {
        let b = if ((1 << i) & bits) == 0 {
            -size
        } else {
            size
        };
        println!("Inserting {} at {} due to {}...", b, i, bits);
        v[i as usize] = b;
    }
    v
}

#[derive(Copy, Clone, Debug)]
pub struct Face {
    pub points: [Vector3<f32>; 4],
    pub normal: Vector3<f32>
}

impl Face {
    fn inside_out(v0 : &Vector3<f32>, v1 : &Vector3<f32>, q : &Point3<f32>, n : &Vector3<f32>) -> bool {
        (v1 - v0).cross(&(q.coords - v0)).dot(n) >= 0.0
    }

    // FIXME: I need to check that the intersection does not happen _behind_ me!
    pub fn intersect(&self, origin: Point3<f32>, dir: Vector3<f32>) -> Option<Point3<f32>> {
        // Compute the intersection point on the supporting plane of the face.
        let dist = (self.points[0] - origin.coords).dot(&self.normal);
        let angle = dir.dot(&self.normal);
        let t = dist/angle;
        let isect = origin + t*dir;
        // We just generalize the standard inside-out test for a triangle here.
        if Face::inside_out(&self.points[0], &self.points[1], &isect, &self.normal)
            && Face::inside_out(&self.points[1], &self.points[3], &isect, &self.normal)
            && Face::inside_out(&self.points[3], &self.points[2], &isect, &self.normal)
            && Face::inside_out(&self.points[2], &self.points[0], &isect, &self.normal) {
            Some(isect)
        } else {
            None
        }
    }
}

pub struct Cube {
    pub faces: Vec<Face>
}

impl Cube {
    pub fn new(dim: u32, size: f32) -> Cube {

        // FIXME: Preallocate with the correct capacity.
        let mut faces = Vec::new();
        // To build a 2-face for an n-cube, we will need to
        // pick 2 dimensions that will vary to form all the corners
        // of the square.
        for d0 in 0..dim {
            for d1 in d0+1..dim {
                // Now that we know what 2 dimensions will vary, we need
                // to pick where on the cube this 2-face will live.

                // For instance, on a 3-cube, if we vary the 'x' and 'y' dimensions, we need to create
                // faces when 'z' is 0 AND 1. To generalize to higher dimensions, we need
                // to generate all possible places where the face can live by looking
                // at all the dimensions that do not vary during face construction.
                //
                // To do this cheaply and easily, we will use some bit level-magic by
                // realizing that an integer 'c < 2 ^ n' can represent a vertex on an
                // n-cube by manner of it's binary representation.
                for loc in 0..2_u32.pow(dim - 2) {
                    let mut v = point(insert_bit(insert_bit(loc, d0), d1), dim, size);

                    v[d0 as usize] = -size;
                    v[d1 as usize] = -size;
                    let bottom_left = linalg::project(&v);

                    v[d0 as usize] = size;
                    v[d1 as usize] = -size;
                    let bottom_right = linalg::project(&v);

                    v[d0 as usize] = -size;
                    v[d1 as usize] = size;
                    let top_left = linalg::project(&v);

                    v[d0 as usize] = size;
                    v[d1 as usize] = size;
                    let top_right = linalg::project(&v);

                    let points = [ bottom_left, bottom_right, top_left, top_right ];

                    let horiz = bottom_right - bottom_left;
                    let vert = top_left - bottom_left;
                    let normal = horiz.cross(&vert);

                    faces.push(Face { points, normal })
                }
            }
        }
        Cube { faces }
    }

    pub fn intersections(&self, origin: Point3<f32>, dir : Vector3<f32>) -> Vec<(Point3<f32>, &Face)> {
        let mut isects : Vec<(Point3<f32>, &Face)> =
        self.faces.iter().filter_map(|face| {
            face.intersect(origin, dir).map(|isect| (isect, face))
        }).collect();
        isects.sort_by_key(|(isect, _)| NotNan::new((origin - isect).norm()).expect("Distance should not be NaN"));
        isects
    }
}
