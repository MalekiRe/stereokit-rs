use stereokit_sys::{pose_identity, pose_t, quat_identity, vec3_zero};
use crate::constants::{QUAT_IDENTITY, VEC3_ZERO};
use crate::values::{Quat, quat_from, Vec3, vec3_from};

pub struct Pose {
	pub(crate)pose: pose_t
}
pub const PoseIdentity: Pose = Pose{
	pose: pose_t{
		position: VEC3_ZERO,
		orientation: QUAT_IDENTITY
	}
};
impl Pose {
	fn new(position: Vec3, orientation: Quat) -> Self {
		Pose {
			pose: pose_t {
				position: vec3_from(position),
				orientation: quat_from(orientation)
			}
		}
	}
}