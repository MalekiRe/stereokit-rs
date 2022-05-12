use std::fmt::Error;
use stereokit_sys::mesh_t;
use crate::values::{Vec3, vec3_from};

pub struct Mesh {
	pub(crate) mesh: mesh_t,
}

impl Mesh {
	pub fn gen_cube(size: Vec3, subdivisions: i32) -> Result<Self, Error>{
		let possible_cube = unsafe {
			stereokit_sys::mesh_gen_cube(vec3_from(size), subdivisions)
		};
		if possible_cube.is_null() {
			return Err(Error);
		}
		Ok(Mesh {mesh: possible_cube})
	}
}
impl Drop for Mesh {
	fn drop(&mut self) {
		unsafe { stereokit_sys::mesh_release(self.mesh) }
	}
}