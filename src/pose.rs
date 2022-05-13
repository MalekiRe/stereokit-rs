use stereokit_sys::pose_t;
use crate::values::{Quat, quat_from, Vec3, vec3_from};

pub struct Pose {
	pose: pose_t
}
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