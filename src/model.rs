use crate::enums::RenderLayer;
use crate::lifecycle::{DrawContext, StereoKitInstanceWrapper};
use crate::material::Material;
use crate::mesh::Mesh;
use crate::pose::Pose;
use crate::shader::Shader;
use crate::values::{color128_from, matrix_from, Color128, Matrix, Vec3};
use crate::StereoKit;
use std::ffi::CString;
use std::fmt::Error;
use std::path::Path;
use std::ptr::NonNull;
use std::rc::{Rc, Weak};
use stereokit_sys::{_model_t, model_draw};
use ustr::ustr;

pub struct Model {
	sk: StereoKitInstanceWrapper,
	pub(crate) model: NonNull<_model_t>,
}
impl Drop for Model {
	fn drop(&mut self) {
		unsafe { stereokit_sys::model_release(self.model.as_ptr()) }
	}
}
impl Model {
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
			model_draw(
				self.model.as_ptr(),
				matrix_from(matrix),
				color128_from(color_linear),
				layer.bits(),
			)
		}
	}
	pub fn from_file(sk: &StereoKit, file_path: &Path, shader: &Shader) -> Option<Self> {
		let file_path = ustr(file_path.as_os_str().to_str().unwrap());
		Some(Model {
			sk: sk.get_wrapper(),
			model: NonNull::new(unsafe {
				stereokit_sys::model_create_file(file_path.as_char_ptr(), shader.shader)
			})?,
		})
	}
}
