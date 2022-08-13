use crate::{
	lifecycle::StereoKitInstance,
	values::{vec3_from, Vec3},
	StereoKit,
};
use std::fmt::Error;
use std::rc::{Rc, Weak};
use stereokit_sys::mesh_t;

pub struct Mesh {
	sk: Weak<StereoKitInstance>,
	pub(crate) mesh: mesh_t,
}

impl Mesh {
	pub fn gen_cube(sk: &StereoKit, size: Vec3, subdivisions: i32) -> Result<Self, Error> {
		let possible_cube = unsafe { stereokit_sys::mesh_gen_cube(vec3_from(size), subdivisions) };
		if possible_cube.is_null() {
			return Err(Error);
		}
		Ok(Mesh {
			sk: sk.get_weak_instance(),
			mesh: possible_cube,
		})
	}
}
impl Drop for Mesh {
	fn drop(&mut self) {
		unsafe { stereokit_sys::mesh_release(self.mesh) }
	}
}
