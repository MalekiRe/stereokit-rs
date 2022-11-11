use std::ops::Deref;
use glam::{EulerRot, Mat4, Quat, Vec3};

#[allow(unused)]
pub mod model;
#[allow(unused)]
pub mod text;
#[allow(unused)]
pub mod math_traits;
#[allow(unused)]
pub mod collider;

pub struct Scale(pub Vec3);

impl Deref for Scale {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec3> for Scale {
    fn from(vec3: Vec3) -> Self {
        Scale(vec3)
    }
}

pub struct Pos(pub Vec3);

impl Deref for Pos {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec3> for Pos {
    fn from(vec3: Vec3) -> Self {
        Pos(vec3)
    }
}

pub fn quat_from_angles(x: f32, y: f32, z: f32) -> Quat {
    Quat::from_euler(EulerRot::XYZ, x.to_radians(), y.to_radians(), z.to_radians())
}

// pub enum Matrix {
//     Mat4(Mat4),
//     Fields {
//         position: Vec3,
//         rotation: Quat,
//         scale: Vec3
//     }
// }