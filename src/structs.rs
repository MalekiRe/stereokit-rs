use crate::material::Material;
use crate::render::RenderLayer;
use crate::values::{vec3_from, MVec3};
use core::fmt::Error;
use std::ffi::{c_void, CString, NulError};
use std::fmt;
use std::fmt::Pointer;
use stereokit_sys::{
	_tex_t, color128, color32, depth_test_, material_set_texture, material_t, mesh_t,
	model_create_mesh, model_draw, model_t, shader_t, sk_settings_t, tex_t,
};
use stereokit_sys::{material_set_float, material_set_queue_offset};

#[derive(Debug, Clone)]
pub struct InitError;

impl fmt::Display for InitError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "invalid init of variable")
	}
}
