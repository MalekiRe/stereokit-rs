use crate::lifecycle::StereoKitInstanceWrapper;
use crate::shader::Shader;
use crate::texture::Texture;
use crate::values::{vec2_from, Color128, Matrix, Vec2, Vec3, Vec4};
use crate::StereoKit;
use num_enum::TryFromPrimitive;
use std::ffi::{c_void, CString};
use std::fmt::Error;
use std::ptr::NonNull;
use std::rc::{Rc, Weak};
use stereokit_sys::{
	_material_t, material_get_shader, material_param__material_param_texture,
	material_param__material_param_vector2, material_param__material_param_vector3,
	material_set_float, material_set_param, material_set_queue_offset, material_set_texture,
};
use ustr::ustr;

pub const DEFAULT_ID_MATERIAL: &str = "default/material";
pub const DEFAULT_ID_MATERIAL_PBR: &str = "default/material_pbr";
pub const DEFAULT_ID_MATERIAL_PBR_CLIP: &str = "default/material_pbr_clip";
pub const DEFAULT_ID_MATERIAL_UNLIT: &str = "default/material_unlit";
pub const DEFAULT_ID_MATERIAL_UNLIT_CLIP: &str = "default/material_unlit_clip";
pub const DEFAULT_ID_MATERIAL_EQUIRECT: &str = "default/equirect_convert";
pub const DEFAULT_ID_MATERIAL_FONT: &str = "default/material_font";
pub const DEFAULT_ID_MATERIAL_HAND: &str = "default/material_hand";
pub const DEFAULT_ID_MATERIAL_UI: &str = "default/material_ui";
pub const DEFAULT_ID_MATERIAL_UI_BOX: &str = "default/material_ui_box";
pub const DEFAULT_ID_MATERIAL_UI_QUADRANT: &str = "default/material_ui_quadrant";

pub trait MaterialParameter {
	const SK_TYPE: u32;

	fn as_raw(&self) -> *const c_void;
}

impl MaterialParameter for Vec2 {
	const SK_TYPE: u32 = material_param__material_param_vector2;

	fn as_raw(&self) -> *const c_void {
		&self as *const _ as *const c_void
	}
}
impl MaterialParameter for Vec3 {
	const SK_TYPE: u32 = material_param__material_param_vector3;

	fn as_raw(&self) -> *const c_void {
		&self as *const _ as *const c_void
	}
}
impl MaterialParameter for Texture {
	const SK_TYPE: u32 = material_param__material_param_texture;

	fn as_raw(&self) -> *const c_void {
		self.tex.as_ptr() as *const c_void
	}
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum Transparency {
	None = 1,
	Blend = 2,
	Add = 3,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum DepthTest {
	Less,
	LessOrEqual,
	Greater,
	GreaterOrEqual,
	Equal,
	NotEqual,
	Always,
	Never,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum Cull {
	Back,
	Front,
	None,
}

pub struct Material {
	pub(crate) sk: StereoKitInstanceWrapper,
	pub material: NonNull<_material_t>,
}
impl Material {
	pub fn create(sk: &StereoKit, shader: &Shader) -> Option<Self> {
		Some(Material {
			sk: sk.get_wrapper(),
			material: NonNull::new(unsafe {
				stereokit_sys::material_create(shader.shader.as_ptr())
			})?,
		})
	}
	pub fn find(id: &str) -> Option<Self> {
		unimplemented!()
	}
	pub fn builtin_copy(&self) -> Option<Self> {
		self.sk.valid().ok()?;
		Some(Material {
			sk: self.sk.clone(),
			material: NonNull::new(unsafe {
				stereokit_sys::material_copy(self.material.as_ptr())
			})?,
		})
	}
	pub fn copy_from_id(sk: &StereoKit, id: &str) -> Option<Self> {
		Some(Material {
			sk: sk.get_wrapper(),
			material: NonNull::new(unsafe {
				stereokit_sys::material_copy_id(ustr(id).as_char_ptr())
			})?,
		})
	}
	pub fn set_id(&self, id: &str) {
		let id = ustr(id);
		unsafe {
			stereokit_sys::material_set_id(self.material.as_ptr(), id.as_char_ptr());
		}
	}
	pub fn set_transparency(&self, mode: Transparency) {
		unsafe { stereokit_sys::material_set_transparency(self.material.as_ptr(), mode as u32) }
	}
	pub fn set_cull(&self, mode: Cull) {
		unsafe { stereokit_sys::material_set_cull(self.material.as_ptr(), mode as u32) }
	}
	pub fn set_wireframe(&self, wireframe: bool) {
		unsafe { stereokit_sys::material_set_wireframe(self.material.as_ptr(), wireframe as i32) }
	}
	pub fn set_depth_test(&self, depth_test_mode: DepthTest) {
		unsafe {
			stereokit_sys::material_set_depth_test(self.material.as_ptr(), depth_test_mode as u32)
		}
	}
	pub fn set_depth_write(&self, write_enabled: bool) {
		unsafe {
			stereokit_sys::material_set_depth_write(self.material.as_ptr(), write_enabled as i32)
		}
	}
	pub fn set_queue_offset(&self, offset: i32) {
		unsafe { material_set_queue_offset(self.material.as_ptr(), offset) }
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
	pub fn has_parameter(&self, name: &str, type_: impl MaterialParameter) -> bool {
		unimplemented!()
	}
	pub fn set_parameter<P>(&self, name: &str, value: &P)
	where
		P: MaterialParameter,
	{
		unsafe {
			material_set_param(
				self.material.as_ptr(),
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
			sk: self.sk.clone(),
			shader: unsafe { NonNull::new(material_get_shader(self.material.as_ptr())).unwrap() },
		}
	}
}
impl Clone for Material {
	fn clone(&self) -> Self {
		let material = unsafe { stereokit_sys::material_copy(self.material.as_ptr()) };
		Self {
			sk: self.sk.clone(),
			material: NonNull::new(material).unwrap(),
		}
	}
}
impl Drop for Material {
	fn drop(&mut self) {
		unsafe { stereokit_sys::material_release(self.material.as_ptr()) }
	}
}
