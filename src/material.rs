use crate::shader::Shader;
use crate::structs::{Cull, DepthTest, Transparency};
use crate::texture::Texture;
use crate::values::{Color128, Matrix, Vec2, Vec3, Vec4};
use crate::StereoKit;
use std::ffi::{c_void, CString};
use std::fmt::Error;
use stereokit_sys::{
	material_get_shader, material_param__material_param_texture, material_set_float,
	material_set_param, material_set_queue_offset, material_set_texture, material_t,
};

pub const DEFAULT_ID_MATERIAL: &'static str = "default/material";
pub const DEFAULT_ID_MATERIAL_PBR: &'static str = "default/material_pbr";
pub const DEFAULT_ID_MATERIAL_PBR_CLIP: &'static str = "default/material_pbr_clip";
pub const DEFAULT_ID_MATERIAL_UNLIT: &'static str = "default/material_unlit";
pub const DEFAULT_ID_MATERIAL_UNLIT_CLIP: &'static str = "default/material_unlit_clip";
pub const DEFAULT_ID_MATERIAL_EQUIRECT: &'static str = "default/equirect_convert";
pub const DEFAULT_ID_MATERIAL_FONT: &'static str = "default/material_font";
pub const DEFAULT_ID_MATERIAL_HAND: &'static str = "default/material_hand";
pub const DEFAULT_ID_MATERIAL_UI: &'static str = "default/material_ui";
pub const DEFAULT_ID_MATERIAL_UI_BOX: &'static str = "default/material_ui_box";
pub const DEFAULT_ID_MATERIAL_UI_QUADRANT: &'static str = "default/material_ui_quadrant";

pub trait MaterialParameter {
	const SK_TYPE: u32;

	fn as_raw(&self) -> *const c_void;
}

impl MaterialParameter for Texture<'_> {
	const SK_TYPE: u32 = material_param__material_param_texture;

	fn as_raw(&self) -> *const c_void {
		self.tex as *const c_void
	}
}

pub struct Material<'a> {
	sk: &'a StereoKit<'a>,
	pub(crate) material: material_t,
}
impl Drop for Material<'_> {
	fn drop(&mut self) {
		unsafe { stereokit_sys::material_release(self.material) }
	}
}
impl<'a> Material<'a> {
	pub fn new(sk: &'a StereoKit, shader: Shader) -> Result<Self, Error> {
		let material = unsafe { stereokit_sys::material_create(shader.shader) };
		if material.is_null() {
			return Err(Error);
		}
		Ok(Material { sk, material })
	}
	pub fn find(id: &str) -> Result<Self, Error> {
		unimplemented!()
	}
	pub fn copy(material: Material) -> Result<Self, Error> {
		unimplemented!()
	}
	pub fn copy_from_id(sk: &'a StereoKit, id: &str) -> Result<Self, Error> {
		let str_id = CString::new(id).unwrap();
		let material = unsafe { stereokit_sys::material_copy_id(str_id.as_ptr()) };
		if material.is_null() {
			return Err(Error);
		}
		Ok(Material { sk, material })
	}
	pub fn set_id(&self, id: &str) {
		let str_id = CString::new(id).unwrap();
		unsafe {
			stereokit_sys::material_set_id(self.material, str_id.as_ptr());
		}
	}
	pub fn set_transparency(&self, mode: Transparency) {
		unimplemented!()
	}
	pub fn set_cull(&self, mode: Cull) {
		unimplemented!()
	}
	pub fn set_wireframe(&self, wireframe: bool) {
		unimplemented!()
	}
	pub fn set_depth_test(&self, depth_test_mode: DepthTest) {
		unimplemented!()
	}
	pub fn set_depth_write(&self, write_enabled: bool) {
		unimplemented!()
	}
	pub fn set_queue_offset(&self, offset: i32) {
		unsafe { material_set_queue_offset(self.material, offset) }
	}
	pub fn get_transparency(&self) -> Transparency {
		unimplemented!()
	}
	pub fn get_cull(&self) -> Cull {
		unimplemented!()
	}
	pub fn get_wireframe(&self) -> bool {
		unimplemented!()
	}
	pub fn get_depth_test(&self) -> DepthTest {
		unimplemented!()
	}
	pub fn get_depth_write(&self) -> bool {
		unimplemented!()
	}
	pub fn get_queue_offset(&self) -> i32 {
		unimplemented!()
	}
	pub fn set_float(&self, name: &str, value: f32) {
		let c_str = CString::new(name).unwrap();
		unsafe { material_set_float(self.material, c_str.as_ptr(), value) }
	}
	pub fn set_vector2(&self, name: &str, value: Vec2) {
		unimplemented!()
	}
	pub fn set_vector3(&self, name: &str, value: Vec3) {
		unimplemented!()
	}
	pub fn set_color(&self, name: &str, value: Color128) {
		unimplemented!()
	}
	pub fn set_vector4(&self, name: &str, value: Vec4) {
		unimplemented!()
	}
	pub fn set_vector(&self, name: &str, value: Vec4) {
		self.set_vector4(name, value);
	}
	pub fn set_int(&self, name: &str, value: i32) {
		unimplemented!()
	}
	pub fn set_int2(&self, name: &str, value1: i32, value2: i32) {
		unimplemented!()
	}
	pub fn set_int3(&self, name: &str, value1: i32, value2: i32, value3: i32) {
		unimplemented!()
	}
	pub fn set_int4(&self, name: &str, value1: i32, value2: i32, value3: i32, value4: i32) {
		unimplemented!()
	}
	pub fn set_matrix(&self, name: &str, value: Matrix) {
		unimplemented!()
	}
	pub fn set_texture(&self, name: &str, value: Texture) {
		let c_str = CString::new(name).unwrap();
		unsafe {
			material_set_texture(self.material, c_str.as_ptr(), value.tex);
		}
	}
	pub fn set_texture_id(&self, id: u64, value: Texture) -> bool {
		unimplemented!()
	}
	pub fn has_parameter(&self, name: &str, type_: impl MaterialParameter) -> bool {
		unimplemented!()
	}
	pub fn set_parameter<P>(&self, name: &str, value: &P)
	where
		P: MaterialParameter,
	{
		unsafe {
			material_set_param(
				self.material,
				ustr::ustr(name).as_char_ptr(),
				P::SK_TYPE,
				value.as_raw(),
			);
		}
	}
	pub fn set_parameter_id(&self, id: u64, type_: impl MaterialParameter, value: c_void) {
		unimplemented!()
	}
	pub fn get_parameter(&self, id: u64, type_: impl MaterialParameter, value: c_void) {
		unimplemented!()
	}
	pub fn get_param_id(&self, id: u64, type_: impl MaterialParameter, out_value: c_void) {
		unimplemented!()
	}
	pub fn get_param_info(
		&self,
		index: i32,
		out_name: Vec<&str>,
		out_type: &mut impl MaterialParameter,
	) {
		unimplemented!()
	}
	pub fn get_param_count(&self) -> i32 {
		unimplemented!()
	}
	pub fn set_shader(&self, shader: Shader) {
		unimplemented!()
	}
	pub fn get_shader(&self) -> Shader {
		Shader {
			sk: self.sk,
			shader: unsafe { material_get_shader(self.material) },
		}
	}
}
