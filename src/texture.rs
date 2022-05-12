use std::fmt::Error;
use stereokit_sys::tex_t;
use crate::values::{color128_from, color128_to, Color32, color32_from, color32_to};

pub struct Texture {
	pub(super) tex: tex_t,
}

impl Drop for Texture {
	fn drop(&mut self) {
		unsafe { stereokit_sys::tex_release(self.tex) }
	}
}
impl Texture {
	pub fn from_color32(data: Color32, width: i32, height: i32, uses_srgb_data: bool) -> Result<Self, Error> {
		let mut my_var: stereokit_sys::bool32_t = 0;
		if uses_srgb_data {
			my_var = 1;
		}
		let texture: tex_t = unsafe {
			stereokit_sys::tex_create_color32(&mut color32_from(data), width, height, my_var)
		};
		if texture.is_null() { return Err(Error);}
		Ok(Texture { tex: texture })
	}
}
