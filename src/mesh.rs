use crate::{
	values::{vec3_from, Vec3},
	StereoKit,
};
use std::fmt::Error;
use stereokit_sys::mesh_t;

pub struct Mesh<'a> {
	sk: &'a StereoKit<'a>,
	pub(crate) mesh: mesh_t,
}

impl<'a> Mesh<'a> {
	pub fn gen_cube(sk: &'a StereoKit, size: Vec3, subdivisions: i32) -> Result<Self, Error> {
		let possible_cube = unsafe { stereokit_sys::mesh_gen_cube(vec3_from(size), subdivisions) };
		if possible_cube.is_null() {
			return Err(Error);
		}
		Ok(Mesh {
			sk,
			mesh: possible_cube,
		})
	}
}
impl Drop for Mesh<'_> {
	fn drop(&mut self) {
		unsafe { stereokit_sys::mesh_release(self.mesh) }
	}
}
