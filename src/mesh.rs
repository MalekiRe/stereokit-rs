use crate::{
	lifecycle::StereoKitDraw,
	material::Material,
	render::RenderLayer,
	values::{matrix_from, vec2_from, vec3_from, Color128, MMatrix, MVec2, MVec3},
	StereoKit,
};
use std::rc::{Rc, Weak};
use std::{fmt::Error, ptr::NonNull};
// use std::error::Report;
use crate::bounds::Bounds;
use crate::lifecycle::StereoKitContext;
use crate::values::vec3_to;
use color_eyre::{Report, Result};
use stereokit_sys::{_mesh_t, bool32_t, mesh_draw};

pub struct Mesh {
	pub(crate) mesh: NonNull<_mesh_t>,
}

impl Mesh {
	pub fn gen_cube(
		_sk: &impl StereoKitContext,
		size: impl Into<MVec3>,
		subdivisions: i32,
	) -> Result<Self> {
		let size = size.into();
		Ok(Mesh {
			mesh: NonNull::new(unsafe {
				stereokit_sys::mesh_gen_cube(vec3_from(size), subdivisions)
			})
			.ok_or(Report::msg(format!(
				"Failed to create a cube from the size '{:?}' and subdivisions '{}'.",
				size, subdivisions
			)))?,
		})
	}
	pub fn gen_plane(
		_sk: &impl StereoKitContext,
		dimensions: impl Into<MVec2>,
		plane_normal: impl Into<MVec3>,
		plane_top_direction: impl Into<MVec3>,
		subdivisions: i32,
	) -> Result<Self> {
		let (dimensions, plane_normal, plane_top_direction) = (
			dimensions.into(),
			plane_normal.into(),
			plane_top_direction.into(),
		);
		Ok(Mesh {
			mesh: NonNull::new(unsafe {
				stereokit_sys::mesh_gen_plane(
					vec2_from(dimensions),
					vec3_from(plane_normal),
					vec3_from(plane_top_direction),
					subdivisions,
					0,
				)
			}).ok_or(Report::msg(format!("Failed to create a plane from dimensions '{:?}' normal '{:?}' top '{:?}' subdivisions '{}'", dimensions, plane_normal, plane_top_direction, subdivisions)))?,
		})
	}
	pub fn draw(
		&self,
		_ctx: &StereoKitDraw,
		material: &Material,
		matrix: MMatrix,
		color_linear: impl Into<Color128>,
		layer: RenderLayer,
	) {
		unsafe {
			mesh_draw(
				self.mesh.as_ptr(),
				material.material.as_ptr(),
				matrix_from(matrix),
				color_linear.into(),
				layer.bits(),
			)
		}
	}
	pub fn get_bounds(&self, _sk: &impl StereoKitContext) -> Bounds {
		let bounds = unsafe { stereokit_sys::mesh_get_bounds(self.mesh.as_ptr()) };
		Bounds::new(vec3_to(bounds.center), vec3_to(bounds.dimensions))
	}
	pub fn mesh_get_keep_data(&self, _sk: &impl StereoKitContext) -> bool {
		unsafe { stereokit_sys::mesh_get_keep_data(self.mesh.as_ptr()) != 0 }
	}
	pub fn mesh_set_keep_data(&mut self, _sk: &impl StereoKitContext, keep_data: bool) {
		unsafe { stereokit_sys::mesh_set_keep_data(self.mesh.as_ptr(), keep_data as bool32_t) }
	}
}
impl Clone for Mesh {
	fn clone(&self) -> Self {
		let mesh = unsafe { stereokit_sys::mesh_copy(self.mesh.as_ptr()) };
		Self {
			mesh: NonNull::new(mesh).unwrap(),
		}
	}
}
impl Drop for Mesh {
	fn drop(&mut self) {
		unsafe { stereokit_sys::mesh_release(self.mesh.as_ptr()) }
	}
}
