use crate::{
	lifecycle::{DrawContext, StereoKitInstanceWrapper},
	material::Material,
	render::RenderLayer,
	values::{color128_from, matrix_from, vec2_from, vec3_from, Color128, Matrix, Vec2, Vec3},
	StereoKit,
};
use std::rc::{Rc, Weak};
use std::{fmt::Error, ptr::NonNull};
use stereokit_sys::{_mesh_t, mesh_draw};

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
	pub fn gen_plane(
		sk: &StereoKit,
		dimensions: Vec2,
		plane_normal: Vec3,
		plane_top_direction: Vec3,
		subdivisions: i32,
	) -> Option<Self> {
		Some(Mesh {
			sk: sk.get_wrapper(),
			mesh: NonNull::new(unsafe {
				stereokit_sys::mesh_gen_plane(
					vec2_from(dimensions),
					vec3_from(plane_normal),
					vec3_from(plane_top_direction),
					subdivisions,
				)
			})?,
		})
	}
	pub fn draw(
		&self,
		_ctx: &DrawContext,
		material: &Material,
		matrix: Matrix,
		color_linear: Color128,
		layer: RenderLayer,
	) {
		unsafe {
			mesh_draw(
				self.mesh.as_ptr(),
				material.material.as_ptr(),
				matrix_from(matrix),
				color128_from(color_linear),
				layer.bits(),
			)
		}
	}
}
impl Clone for Mesh {
	fn clone(&self) -> Self {
		let mesh = unsafe { stereokit_sys::mesh_copy(self.mesh.as_ptr()) };
		Self {
			sk: self.sk.clone(),
			mesh: NonNull::new(mesh).unwrap(),
		}
	}
}
impl Drop for Mesh {
	fn drop(&mut self) {
		unsafe { stereokit_sys::mesh_release(self.mesh.as_ptr()) }
	}
}
