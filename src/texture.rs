use crate::render::SphericalHarmonics;
use crate::values::{color128_from, color128_to, color32_from, color32_to, Color32};
use crate::StereoKit;
use std::ffi::CString;
use std::fmt::Error;
use stereokit_sys::tex_t;

pub struct Texture<'a> {
	sk: &'a StereoKit<'a>,
	pub(super) tex: tex_t,
}

impl Drop for Texture<'_> {
	fn drop(&mut self) {
		unsafe { stereokit_sys::tex_release(self.tex) }
	}
}
impl<'a> Texture<'a> {
	pub fn from_color32(
		sk: &'a StereoKit,
		data: Color32,
		width: i32,
		height: i32,
		uses_srgb_data: bool,
	) -> Result<Self, Error> {
		let mut my_var: stereokit_sys::bool32_t = 0;
		if uses_srgb_data {
			my_var = 1;
		}
		let texture: tex_t = unsafe {
			stereokit_sys::tex_create_color32(&mut color32_from(data), width, height, my_var)
		};
		if texture.is_null() {
			return Err(Error);
		}
		Ok(Texture { sk, tex: texture })
	}

	pub fn from_cubemap_equirectangular(
		sk: &'a StereoKit,
		file_path: &str,
		uses_srgb_data: bool,
		load_priority: i32,
	) -> Result<(Self, SphericalHarmonics), Error> {
		let c_file_path = CString::new(file_path).unwrap();
		let mut spherical_harmonics = stereokit_sys::spherical_harmonics_t {
			coefficients: [unsafe { stereokit_sys::vec3_zero }; 9],
		};
		let tex = unsafe {
			stereokit_sys::tex_create_cubemap_file(
				c_file_path.as_ptr(),
				uses_srgb_data.into(),
				&mut spherical_harmonics,
				load_priority,
			)
		};
		Ok((
			Texture { sk, tex },
			SphericalHarmonics {
				spherical_harmonics,
			},
		))
	}
}
