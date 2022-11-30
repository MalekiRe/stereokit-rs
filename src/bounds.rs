use std::ptr::NonNull;
use crate::values::{Vec3, vec3_from};
use stereokit_sys::{bounds_t, vec3};
pub struct Bounds {
    pub center: Vec3,
    pub dimensions: Vec3,
}

impl Bounds {
    pub fn bounds_point_contains(&self, pt: Vec3) -> bool {
        unsafe {
            stereokit_sys::bounds_point_contains(self.as_bounds(), vec3_from(pt)) != 0
        }
    }
    fn as_bounds(&self) -> bounds_t {
        bounds_t{
            center: vec3_from(self.center),
            dimensions: vec3_from(self.dimensions)
        }
    }
    pub fn new(center: impl Into<Vec3>, dimensions: impl Into<Vec3>) -> Self {
        Self {
            center: center.into(),
            dimensions: dimensions.into()
        }
    }
    pub fn bounds_capsule_contains(&self, pt1: Vec3, pt2: Vec3, radius: f32) -> bool {
        unsafe {
            stereokit_sys::bounds_capsule_contains(self.as_bounds(), vec3_from(pt1), vec3_from(pt2), radius) != 0
        }
    }
}