use std::ffi::CString;
use stereokit_sys::{ui_move_, ui_win_};
use crate::values::{Vec2, vec2_from};

pub enum WindowType {
	WindowEmpty = 0,
	WindowHead = 1,
	WindowBody = 2,
	WindowNormal = 3
}
pub enum MoveType {
	MoveExact = 0,
	MoveFaceUser = 1,
	MovePosOnly = 2,
	MoveNone = 3
}
pub fn begin(window_title: &str, mut pose: crate::pose::Pose, size: Vec2, window_type: WindowType, move_type: MoveType) {
	let my_c_string = CString::new(window_title).unwrap();
	unsafe {stereokit_sys::ui_window_begin(my_c_string.as_ptr(), &mut pose.pose, vec2_from(size), window_type as ui_win_, move_type as ui_move_)};
}
pub fn end() {
	unsafe {stereokit_sys::ui_window_end();}
}