use crate::bounds::Bounds;
use crate::lifecycle::{StereoKitContext, StereoKitDraw};
use crate::material::Material;
use crate::mesh::Mesh;
use crate::pose::Pose;
use crate::render::RenderLayer;
use crate::shader::Shader;
use crate::values::{matrix_from, vec3_from, vec3_to, Color128, MMatrix, MVec3};
use crate::StereoKit;
use color_eyre::{Report, Result};
use std::ffi::{c_void, CString};
use std::fmt::Error;
use std::path::Path;
use std::ptr::{null, null_mut, NonNull};
use std::rc::{Rc, Weak};
use stereokit_sys::_model_t;
use ustr::ustr;

pub struct Model {
	pub(crate) model: NonNull<_model_t>,
}
impl Model {
	pub fn from_file(
		_sk: &impl StereoKitContext,
		file_path: impl AsRef<Path>,
		shader: Option<&Shader>,
	) -> Result<Self> {
		let file_path = ustr(
			file_path
				.as_ref()
				.as_os_str()
				.to_str()
				.ok_or(Report::msg("failed string conversion in Model::from_file"))?,
		);
		Ok(Model {
			model: NonNull::new(unsafe {
				stereokit_sys::model_create_file(
					file_path.as_char_ptr(),
					shader
						.map(|shader| shader.shader.as_ptr())
						.unwrap_or(null_mut()),
				)
			})
			.ok_or(Report::msg(format!(
				"Unable to create model from file path '{}'.",
				file_path
			)))?,
		})
	}
	pub fn from_mem(
		_sk: &impl StereoKitContext,
		file_name: &str,
		memory: &[u8],
		shader: Option<&Shader>,
	) -> Result<Self> {
		let file_name = ustr(file_name);
		Ok(Model {
			model: NonNull::new(unsafe {
				stereokit_sys::model_create_mem(
					file_name.as_char_ptr(),
					memory.as_ptr() as *mut c_void,
					memory.len(),
					shader
						.map(|shader| shader.shader.as_ptr())
						.unwrap_or(null_mut()),
				)
			})
			.ok_or(Report::msg(format!(
				"Unable to create model '{}' from memory",
				file_name
			)))?,
		})
	}
	pub fn from_mesh(
		_sk: &impl StereoKitContext,
		mesh: &Mesh,
		material: &Material,
	) -> Result<Self> {
		Ok(Model {
			model: NonNull::new(unsafe {
				stereokit_sys::model_create_mesh(mesh.mesh.as_ptr(), material.material.as_ptr())
			})
			.ok_or(Report::msg("Failed to create model from mesh and material"))?,
		})
	}
	pub fn draw(
		&self,
		_ctx: &StereoKitDraw,
		matrix: MMatrix,
		color_linear: impl Into<Color128>,
		layer: RenderLayer,
	) {
		unsafe {
			stereokit_sys::model_draw(
				self.model.as_ptr(),
				matrix_from(matrix),
				color_linear.into(),
				layer.bits(),
			)
		}
	}
	pub fn get_material(&self, _sk: &impl StereoKitContext, subset: i32) -> Option<Material> {
		Some(Material {
			material: NonNull::new(unsafe {
				stereokit_sys::model_get_material(self.model.as_ptr(), subset)
			})?,
		})
	}
	pub fn set_material(&self, _sk: &impl StereoKitContext, subset: i32, material: &Material) {
		unsafe {
			stereokit_sys::model_set_material(
				self.model.as_ptr(),
				subset,
				material.material.as_ptr(),
			);
		}
	}
	pub fn get_bounds(&self, _sk: &impl StereoKitContext) -> Bounds {
		let b = unsafe { stereokit_sys::model_get_bounds(self.model.as_ptr()) };
		Bounds::new(vec3_to(b.center), vec3_to(b.dimensions))
	}
}
impl Clone for Model {
	fn clone(&self) -> Self {
		let model = unsafe { stereokit_sys::model_copy(self.model.as_ptr()) };
		Self {
			model: NonNull::new(model).unwrap(),
		}
	}
}
impl Drop for Model {
	fn drop(&mut self) {
		unsafe { stereokit_sys::model_release(self.model.as_ptr()) }
	}
}
