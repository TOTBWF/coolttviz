use std::f32::consts::PI;

use nalgebra::{Isometry3, Vector3, Point3};

// TODO: It might be a good idea to cache the eye and view?
pub struct Camera {
    radius: f32,
    polar: f32,
    azimuth: f32,
}

impl Camera {
    pub fn new() -> Camera {
        let azimuth = 90.0_f32.to_radians();
        let polar = 0.0;
        let radius = 4.0;
        Camera {
            radius,
            polar,
            azimuth,
        }
    }

    pub fn rotate_azimuth(&mut self, delta: f32) {
        self.azimuth += delta;
    }

    pub fn rotate_polar(&mut self, delta: f32) {
        self.polar += delta;

        if self.polar < - PI/2.0 {
            self.polar = -PI/2.0;
        } else if self.polar > PI/2.0 {
            self.polar = PI/2.0;
        }
    }

    pub fn zoom(&mut self, delta: f32) {
        self.radius += delta;
        if self.radius < 0.1 {
            self.radius = 0.1;
        }
    }

    pub fn eye(&self) -> Point3<f32> {
        Point3::new(
            self.radius * self.polar.cos() * self.azimuth.cos(),
            self.radius * self.polar.sin(),
            self.radius * self.polar.cos() * self.azimuth.sin()
        )
    }

    pub fn view(&self) -> Isometry3<f32> {
        let origin : Point3<f32> = Point3::new(0.0, 0.0, 0.0);
        let up : Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
        Isometry3::look_at_rh(&self.eye(), &origin, &up)
    }
}
