use crate::values::{vec3_from, MVec3};
use std::ptr::NonNull;
use stereokit_sys::{bounds_t, vec3};
pub struct Bounds {
	pub center: MVec3,
	pub dimensions: MVec3,
}

impl Bounds {
	pub fn bounds_point_contains(&self, pt: MVec3) -> bool {
		unsafe { stereokit_sys::bounds_point_contains(self.as_bounds(), vec3_from(pt)) != 0 }
	}
	fn as_bounds(&self) -> bounds_t {
		bounds_t {
			center: vec3_from(self.center),
			dimensions: vec3_from(self.dimensions),
		}
	}
	pub fn new(center: impl Into<MVec3>, dimensions: impl Into<MVec3>) -> Self {
		Self {
			center: center.into(),
			dimensions: dimensions.into(),
		}
	}
	pub fn bounds_capsule_contains(&self, pt1: MVec3, pt2: MVec3, radius: f32) -> bool {
		unsafe {
			stereokit_sys::bounds_capsule_contains(
				self.as_bounds(),
				vec3_from(pt1),
				vec3_from(pt2),
				radius,
			) != 0
		}
	}
}
