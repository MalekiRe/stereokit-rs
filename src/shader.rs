use crate::{lifecycle::StereoKitInstanceWrapper, StereoKit};
use std::{
	ffi::c_void,
	path::Path,
	ptr::NonNull,
	rc::{Rc, Weak},
};
use stereokit_sys::{_shader_t, font_find, shader_find};
use ustr::ustr;
use crate::font::Font;

pub struct Shader {
	pub(crate) sk: StereoKitInstanceWrapper,
	pub(crate) shader: NonNull<_shader_t>,
}

impl Shader {
	pub fn from_file(sk: &StereoKit, file_path: impl AsRef<Path>, shader: &Shader) -> Option<Self> {
		let file_path = ustr(file_path.as_ref().as_os_str().to_str()?);
		Some(Shader {
			sk: sk.get_wrapper(),
			shader: NonNull::new(unsafe {
				stereokit_sys::shader_create_file(file_path.as_char_ptr())
			})?,
		})
	}
	pub fn from_mem(sk: &StereoKit, memory: &[u8]) -> Option<Self> {
		Some(Shader {
			sk: sk.get_wrapper(),
			shader: NonNull::new(unsafe {
				stereokit_sys::shader_create_mem(
					memory.as_ptr() as *mut c_void,
					memory.len() as u64,
				)
			})?,
		})
	}
	pub fn default(sk: &StereoKit) -> Self {
		let default_id = ustr("default/shader");
		Shader {
			sk: sk.get_wrapper(),
			shader: NonNull::new(unsafe { shader_find(default_id.as_char_ptr()) }).unwrap(),
		}
	}
	pub fn p_b_r(sk: &StereoKit) -> Self {
		Shader::from_name(sk, "default/shader_pbr").unwrap()
	}
	pub fn from_name(sk: &StereoKit, name: &str) -> Option<Self> {
		Some(Shader {
			sk: sk.get_wrapper(),
			shader: NonNull::new(unsafe { shader_find(ustr(name).as_char_ptr()) })?,
		})
	}
}
impl Drop for Shader {
	fn drop(&mut self) {
		unsafe { stereokit_sys::shader_release(self.shader.as_ptr()) };
	}
}
