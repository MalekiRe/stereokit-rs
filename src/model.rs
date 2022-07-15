use std::ffi::CString;
use std::fmt::Error;
use std::path::Path;
use stereokit_sys::{model_draw, model_t};
use crate::enums::RenderLayer;
use crate::material::Material;
use crate::mesh::Mesh;
use crate::pose::Pose;
use crate::shader::Shader;
use crate::values::{Color128, color128_from, Matrix, Vec3};

pub struct Model {
	pub(crate) model: model_t
}
unsafe impl Send for Model {

}
unsafe impl Sync for Model {

}
impl Drop for Model {
	fn drop(&mut self) {
		unsafe {stereokit_sys::model_release(self.model)}
	}
}
impl Model {
	pub fn from_mesh(mesh: Mesh, material: Material) -> Result<Self, Error> {
		let possible_model = unsafe {
			stereokit_sys::model_create_mesh(mesh.mesh, material.material)
		};
		if possible_model.is_null() { return Err(Error); }
		Ok(Model{model: possible_model})
	}
	pub fn draw(&self, matrix: Matrix, color_linear: Color128, layer: RenderLayer) {
		unsafe {model_draw(self.model, matrix.matrix, color128_from(color_linear), layer as i32)}
	}
	pub fn from_file(file_path: &Path, shader: Shader) -> Result<Self, Error> {
		let my_str = CString::new(file_path.as_os_str().to_str().unwrap()).unwrap();
		println!("the path is: {}",my_str.to_str().unwrap());
		let possible_model = unsafe{
			stereokit_sys::model_create_file(my_str.as_ptr(), shader.shader)
		};
		if possible_model.is_null() { return Err(Error); }
		Ok(Model{model: possible_model})
	}
}