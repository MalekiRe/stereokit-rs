use std::ffi::CString;
use stereokit_sys::{bool32_t, text_align_, ui_label, ui_sameline, ui_settings, ui_space, ui_text};
use crate::enums::TextAlign;

pub fn sameline() {
	unsafe{ui_sameline()}
}
pub fn space(space: f32) {
	unsafe {ui_space(space)}
}

pub fn text(text: &str, text_align: TextAlign) {
	let my_string = CString::new(text).unwrap();
	unsafe {
		ui_text(my_string.as_ptr(), text_align as text_align_)
	}
}
pub fn label(text: &str, use_padding: bool) {
	let my_string = CString::new(text).unwrap();
	unsafe {
		ui_label(my_string.as_ptr(), use_padding as bool32_t)
	}
}
pub type UISettings = stereokit_sys::ui_settings_t;
pub fn settings(settings: UISettings) {
	unsafe {ui_settings(settings);}
}