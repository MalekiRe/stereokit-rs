use glam::{EulerRot, Mat4, Quat, Vec3};

#[allow(unused)]
pub mod model;

pub fn quat_from_angles(x: f32, y: f32, z: f32) -> Quat {
    Quat::from_euler(EulerRot::XYZ, x, y, z)
}

// pub enum Matrix {
//     Mat4(Mat4),
//     Fields {
//         position: Vec3,
//         rotation: Quat,
//         scale: Vec3
//     }
// }