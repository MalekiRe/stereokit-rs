use crate::lifecycle::{DrawContext, StereoKitInstanceWrapper};
use crate::material::Material;
use crate::mesh::Mesh;
use crate::pose::Pose;
use crate::render::RenderLayer;
use crate::shader::Shader;
use crate::values::{color128_from, matrix_from, Color128, Matrix, Vec3};
use crate::StereoKit;
use std::ffi::{c_void, CString};
use std::fmt::Error;
use std::path::Path;
use std::ptr::{null, null_mut, NonNull};
use std::rc::{Rc, Weak};
use stereokit_sys::_model_t;
use ustr::ustr;

#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Component))]
pub struct Model {
	sk: StereoKitInstanceWrapper,
	pub(crate) model: NonNull<_model_t>,
}
impl Model {
	pub fn from_file(sk: &StereoKit, file_path: &Path, shader: Option<&Shader>) -> Option<Self> {
		let file_path = ustr(file_path.as_os_str().to_str().unwrap());
		Some(Model {
			sk: sk.get_wrapper(),
			model: NonNull::new(unsafe {
				stereokit_sys::model_create_file(
					file_path.as_char_ptr(),
					shader
						.map(|shader| shader.shader.as_ptr())
						.unwrap_or(null_mut()),
				)
			})?,
		})
	}
	pub fn from_mem(
		sk: &StereoKit,
		file_name: &str,
		memory: &[u8],
		shader: Option<&Shader>,
	) -> Option<Self> {
		let file_name = ustr(file_name);
		Some(Model {
			sk: sk.get_wrapper(),
			model: NonNull::new(unsafe {
				stereokit_sys::model_create_mem(
					file_name.as_char_ptr(),
					memory.as_ptr() as *mut c_void,
					memory.len() as u64,
					shader
						.map(|shader| shader.shader.as_ptr())
						.unwrap_or(null_mut()),
				)
			})?,
		})
	}
	pub fn from_mesh(sk: &StereoKit, mesh: &Mesh, material: &Material) -> Option<Self> {
		Some(Model {
			sk: sk.get_wrapper(),
			model: NonNull::new(unsafe {
				stereokit_sys::model_create_mesh(mesh.mesh.as_ptr(), material.material.as_ptr())
			})?,
		})
	}
	pub fn draw(
		&self,
		_ctx: &DrawContext,
		matrix: Matrix,
		color_linear: Color128,
		layer: RenderLayer,
	) {
		unsafe {
			stereokit_sys::model_draw(
				self.model.as_ptr(),
				matrix_from(matrix),
				color128_from(color_linear),
				layer.bits(),
			)
		}
	}
	pub fn get_material(&self, sk: &StereoKit, subset: i32) -> Option<Material> {
		Some(Material {
			sk: sk.get_wrapper(),
			material: NonNull::new(unsafe {
				stereokit_sys::model_get_material(self.model.as_ptr(), subset)
			})?,
		})
	}
	pub fn set_material(&self, subset: i32, material: &Material) {
		unsafe {
			stereokit_sys::model_set_material(
				self.model.as_ptr(),
				subset,
				material.material.as_ptr(),
			);
		}
	}
}
impl Clone for Model {
	fn clone(&self) -> Self {
		let model = unsafe { stereokit_sys::model_copy(self.model.as_ptr()) };
		Self {
			sk: self.sk.clone(),
			model: NonNull::new(model).unwrap(),
		}
	}
}
impl Drop for Model {
	fn drop(&mut self) {
		unsafe { stereokit_sys::model_release(self.model.as_ptr()) }
	}
}
