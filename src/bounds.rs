use std::ptr::NonNull;
use crate::lifecycle::StereoKitInstanceWrapper;
use crate::values::{Vec3, vec3_from};
use stereokit_sys::bounds_t;
pub struct Bounds {
    pub(crate) sk: StereoKitInstanceWrapper,
    pub(crate) bounds: bounds_t,
}

impl Bounds {
    pub fn bounds_point_contains(&self, pt: Vec3) -> bool {
        unsafe {
            stereokit_sys::bounds_point_contains(self.bounds, vec3_from(pt)) != 0
        }
    }
}