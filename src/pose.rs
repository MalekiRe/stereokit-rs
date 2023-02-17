use crate::values::{matrix_to, quat_from, vec3_from, MMatrix, MQuat, MVec3, pose_from};
use std::fmt::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
	pub position: MVec3,
	pub orientation: MQuat,
}
impl Pose {
	pub const IDENTITY: Pose = Pose {
		position: MVec3 {
			x: 0.,
			y: 0.,
			z: 0.,
		},
		orientation: MQuat {
			v: MVec3 {
				x: 0.,
				y: 0.,
				z: 0.,
			},
			s: 1.,
		},
	};
	pub fn new(position: impl Into<MVec3>, orientation: impl Into<MQuat>) -> Self {
		Pose {
			position: position.into(),
			orientation: orientation.into(),
		}
	}
	pub fn pose_matrix(&self, vec3: impl Into<MVec3>) -> MMatrix {
		unsafe {
			matrix_to(stereokit_sys::pose_matrix(
				&pose_from(*self),
				vec3.into().into(),
			))
		}
	}
	pub fn as_matrix(&self) -> MMatrix {
		self.pose_matrix(MVec3::from([1.0, 1.0, 1.0]))
	}
}
