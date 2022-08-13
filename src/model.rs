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
use std::rc::{Rc, Weak};
use stereokit_sys::{model_draw, model_t};

pub struct Model {
	sk: StereoKitInstanceWrapper,
	pub(crate) model: model_t,
}
impl Drop for Model {
	fn drop(&mut self) {
		unsafe { stereokit_sys::model_release(self.model) }
	}
}
impl Model {
	pub fn from_mesh(sk: &StereoKit, mesh: &Mesh, material: &Material) -> Result<Self, Error> {
		let possible_model =
			unsafe { stereokit_sys::model_create_mesh(mesh.mesh, material.material) };
		if possible_model.is_null() {
			return Err(Error);
		}
		Ok(Model {
			sk: sk.get_wrapper(),
			model: possible_model,
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
				self.model,
				matrix_from(matrix),
				color128_from(color_linear),
				layer.bits(),
			)
		}
	}
	pub fn from_file(sk: &StereoKit, file_path: &Path, shader: &Shader) -> Result<Self, Error> {
		let my_str = CString::new(file_path.as_os_str().to_str().unwrap()).unwrap();
		println!("the path is: {}", my_str.to_str().unwrap());
		let possible_model =
			unsafe { stereokit_sys::model_create_file(my_str.as_ptr(), shader.shader) };
		if possible_model.is_null() {
			return Err(Error);
		}
		Ok(Model {
			sk: sk.get_wrapper(),
			model: possible_model,
		})
	}
}
