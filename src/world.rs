use crate::lifecycle::StereoKitContext;
use crate::pose::Pose;
use crate::values::{quat_to, vec3_to};

pub struct World {}

impl World {
	pub fn has_bounds(_sk: &impl StereoKitContext) -> bool {
		unsafe { stereokit_sys::world_has_bounds() != 0 }
	}
	pub fn get_bounds_pose(sk: &impl StereoKitContext) -> Option<Pose> {
		if !Self::has_bounds(sk) {
			return None;
		}
		let pose = unsafe { stereokit_sys::world_get_bounds_pose() };
		Some(Pose {
			position: vec3_to(pose.position),
			orientation: quat_to(pose.orientation),
		})
	}
}
