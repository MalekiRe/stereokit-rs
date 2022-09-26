use std::fmt::Error;
use std::path::Path;
use std::{ffi::CString, ptr::NonNull};
use stereokit_sys::{_font_t, default_id_font, font_create, font_find};
use ustr::ustr;

use crate::{lifecycle::StereoKitInstanceWrapper, StereoKit};
#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Component))]
pub struct Font {
	sk: StereoKitInstanceWrapper,
	pub(crate) font: NonNull<_font_t>,
}

impl Font {
	pub fn from_file(sk: &StereoKit, file: &Path) -> Option<Self> {
		let file_path = ustr(file.as_os_str().to_str()?);

		Some(Font {
			sk: sk.get_wrapper(),
			font: NonNull::new(unsafe { stereokit_sys::font_create(file_path.as_char_ptr()) })?,
		})
	}
	pub fn default(sk: &StereoKit) -> Self {
		let default_id = ustr("default/font");

		Font {
			sk: sk.get_wrapper(),
			font: NonNull::new(unsafe { font_find(default_id.as_char_ptr()) }).unwrap(),
		}
	}
}
