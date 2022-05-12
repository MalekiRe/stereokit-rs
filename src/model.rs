use std::fmt::Error;
use stereokit_sys::{model_draw, model_t};
use crate::enums::RenderLayer;
use crate::material::Material;
use crate::mesh::Mesh;
use crate::values::Matrix;

pub struct Model {
	model: model_t
}
impl Drop for Model {
	fn drop(&mut self) {
		unsafe {stereokit_sys::model_release(self.model)}
	}
}
impl Model {
	pub fn from_mesh(mesh: Mesh, material: Material) -> Result<Self, Error> {
		let possible_model = unsafe {stereokit_sys::model_create_mesh(mesh.mesh, material.material)};
		if possible_model.is_null() {
			return Err(Error);
		}
		Ok(Model{model: possible_model})
	}
	pub fn draw(&self, matrix: Matrix, color_linear: Color128, layer: RenderLayer) {
		unsafe {model_draw(self.model, matrix.matrix, color_linear.to_color128(), layer as u32)}
	}
}