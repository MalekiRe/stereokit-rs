use std::ffi::CString;
use std::fmt::Error;
use std::path::Path;
use std::rc::{Rc, Weak};
use stereokit_sys::{_font_t, default_id_font, font_create, font_find, font_t};

use crate::lifecycle::StereoKitInstance;
use crate::StereoKit;

pub struct Font {
	sk: Weak<StereoKitInstance>,
	pub(crate) font: font_t,
}

impl Font {
	pub fn from_file(sk: &StereoKit, file: &Path) -> Result<Self, Error> {
		let my_string = CString::new(file.as_os_str().to_str().unwrap()).unwrap();
		let possible_font = unsafe { stereokit_sys::font_create(my_string.as_ptr()) };
		if possible_font.is_null() {
			return Err(Error);
		}
		Ok(Font {
			sk: sk.get_weak_instance(),
			font: possible_font,
		})
	}
	pub fn default(sk: &StereoKit) -> Self {
		let my_string = CString::new("default/font").unwrap();
		unsafe {
			Font {
				sk: sk.get_weak_instance(),
				font: font_find(my_string.as_ptr()),
			}
		}
	}
}
