use crate::constants::{QUAT_IDENTITY, VEC3_ZERO};
use crate::values::{matrix_to, quat_from, vec3_from, Matrix, Quat, Vec3};
use std::fmt::Error;
use stereokit_sys::{pose_identity, pose_matrix, pose_t, quat_identity, vec3_zero};

#[derive(Debug, Clone, Copy)]
pub struct Pose {
	pub position: Vec3,
	pub orientation: Quat,
}
impl Pose {
	pub const IDENTITY: Pose = Pose {
		position: Vec3 {
			x: 0.,
			y: 0.,
			z: 0.,
		},
		orientation: Quat {
			v: Vec3 {
				x: 0.,
				y: 0.,
				z: 0.,
			},
			s: 1.,
		},
	};
	pub fn new(position: Vec3, orientation: Quat) -> Self {
		Pose {
			position: position,
			orientation: orientation,
		}
	}
	pub fn pose_matrix(&self, vec3: Vec3) -> Matrix {
		unsafe {
			matrix_to(stereokit_sys::pose_matrix(
				self as *const _ as *const pose_t,
				vec3_from(vec3),
			))
		}
	}
	pub fn as_matrix(&self) -> Matrix {
		self.pose_matrix(Vec3::from([1.0, 1.0, 1.0]))
	}
}
