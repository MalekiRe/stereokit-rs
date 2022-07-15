use std::fmt::Error;
use crate::constants::{QUAT_IDENTITY, VEC3_ZERO};
use crate::values::{quat_from, vec3_from, Quat, Vec3, Matrix};
use stereokit_sys::{pose_identity, pose_matrix, pose_t, quat_identity, vec3_zero};

pub struct Pose {
	pub(crate) pose: pose_t,
}
pub const IDENTITY: Pose = Pose {
	pose: pose_t {
		position: VEC3_ZERO,
		orientation: QUAT_IDENTITY,
	},
};
impl Pose {
	pub fn new(position: Vec3, orientation: Quat) -> Self {
		Pose {
			pose: pose_t {
				position: vec3_from(position),
				orientation: quat_from(orientation),
			},
		}
	}
	pub fn pose_matrix(&self, vec3: Vec3) -> Matrix {
		Matrix{matrix:unsafe {
			stereokit_sys::pose_matrix(&self.pose, vec3_from(vec3))
		}}
	}
	pub fn as_matrix(&self) -> Matrix {
		self.pose_matrix(Vec3::from([1.0, 1.0, 1.0]))
	}
}