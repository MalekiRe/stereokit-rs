use std::{
	ffi::c_void,
	ptr::NonNull,
	rc::{Rc, Weak},
};
use stereokit_sys::_shader_t;

use crate::{lifecycle::StereoKitInstanceWrapper, StereoKit};
pub struct Shader {
	pub(crate) sk: StereoKitInstanceWrapper,
	pub(crate) shader: NonNull<_shader_t>,
}

impl Shader {
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
