use crate::font::Font;
use crate::lifecycle::StereoKitContext;
use color_eyre::eyre::{ContextCompat, ErrReport};
use color_eyre::{Help, Report, Result};
use std::{
	ffi::c_void,
	path::Path,
	ptr::NonNull,
	rc::{Rc, Weak},
};
use stereokit_sys::{_shader_t, font_find, shader_find};
use ustr::ustr;

pub struct Shader {
	pub(crate) shader: NonNull<_shader_t>,
}

impl Shader {
	pub fn from_file(sk: &impl StereoKitContext, file_path: impl AsRef<Path>) -> Result<Self> {
		let file_path = ustr(
			file_path
				.as_ref()
				.as_os_str()
				.to_str()
				.ok_or(Report::msg("file to string conversion failed"))?,
		);
		Ok(Shader {
			shader: NonNull::new(unsafe {
				stereokit_sys::shader_create_file(file_path.as_char_ptr())
			})
			.ok_or(Report::msg(format!(
				"The Shader from file '{}' could not be loaded",
				file_path
			)))?,
		})
	}
	pub fn from_mem(sk: &impl StereoKitContext, memory: &[u8]) -> Result<Self> {
		Ok(Shader {
			shader: NonNull::new(unsafe {
				stereokit_sys::shader_create_mem(
					memory.as_ptr() as *mut c_void,
					memory.len() as u64,
				)
			})
			.ok_or(Report::msg("Unable to create shader from memory"))?,
		})
	}
	pub fn default(sk: &impl StereoKitContext) -> Self {
		let default_id = ustr("default/shader");
		Shader {
			shader: NonNull::new(unsafe { shader_find(default_id.as_char_ptr()) }).unwrap(),
		}
	}
	pub fn p_b_r(sk: &impl StereoKitContext) -> Self {
		Shader::from_name(sk, "default/shader_pbr").unwrap()
	}
	pub fn from_name(sk: &impl StereoKitContext, name: &str) -> Result<Self> {
		Ok(Shader {
			shader: NonNull::new(unsafe { shader_find(ustr(name).as_char_ptr()) })
				.ok_or(Report::msg(format!(
					"The Shader '{}' could not be located from it's name",
					name
				)))
				.suggestion("Try something in the format of 'default/shader_pbr'")?,
		})
	}
}
impl Drop for Shader {
	fn drop(&mut self) {
		unsafe { stereokit_sys::shader_release(self.shader.as_ptr()) };
	}
}
