use std::ffi::CString;
use std::fmt::Error;
use std::path::Path;
use stereokit_sys::font_t;

pub struct Font {
	font: font_t
}

impl Font {
	fn from_file(file: &Path) -> Result<Self, Error> {
		let my_string = CString::new(file.as_os_str().to_str().unwrap()).unwrap();
		let possible_font = unsafe{
			stereokit_sys::font_create(my_string.as_ptr())
		};
		if possible_font.is_null() {
			return Err(Error);
		}
		Ok(Font {font: possible_font})
	}
}