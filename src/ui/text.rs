use std::ffi::CString;
use stereokit_sys::{text_align_, ui_text};
use crate::enums::TextAlign;

pub fn create(text: &str, text_align: TextAlign) {
	let my_string = CString::new(text).unwrap();
	unsafe {
		ui_text(my_string.as_ptr(), text_align as text_align_)
	}
}