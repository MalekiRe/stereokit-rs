use crate::{lifecycle::StereoKitInstanceWrapper, StereoKit};
use std::{
	ffi::c_void,
	path::Path,
	ptr::NonNull,
	rc::{Rc, Weak},
};
use stereokit_sys::_shader_t;
use ustr::ustr;

#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Component))]
pub struct Shader {
	pub(crate) sk: StereoKitInstanceWrapper,
	pub(crate) shader: NonNull<_shader_t>,
}

impl Shader {
	pub fn from_file(sk: &StereoKit, file_path: &Path, shader: &Shader) -> Option<Self> {
		let file_path = ustr(file_path.as_os_str().to_str().unwrap());
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
}
impl Drop for Shader {
	fn drop(&mut self) {
		unsafe { stereokit_sys::shader_release(self.shader.as_ptr()) };
	}
}
