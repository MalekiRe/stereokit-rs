use crate::{
	lifecycle::StereoKitInstanceWrapper,
	values::{vec3_from, Vec3},
	StereoKit,
};
use std::rc::{Rc, Weak};
use std::{fmt::Error, ptr::NonNull};
use stereokit_sys::_mesh_t;

pub struct Mesh {
	sk: StereoKitInstanceWrapper,
	pub(crate) mesh: NonNull<_mesh_t>,
}

impl Mesh {
	pub fn gen_cube(sk: &StereoKit, size: Vec3, subdivisions: i32) -> Option<Self> {
		Some(Mesh {
			sk: sk.get_wrapper(),
			mesh: NonNull::new(unsafe {
				stereokit_sys::mesh_gen_cube(vec3_from(size), subdivisions)
			})?,
		})
	}
}
impl Drop for Mesh {
	fn drop(&mut self) {
		unsafe { stereokit_sys::mesh_release(self.mesh.as_ptr()) }
	}
}
