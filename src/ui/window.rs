use crate::{
	lifecycle::DrawContext,
	textstyle::TextAlign,
	values::{vec2_from, Vec2},
};
use std::{ffi::CString, marker::PhantomData};
use stereokit_sys::{
	bool32_t, pose_t, text_align_, ui_label, ui_move_, ui_sameline, ui_settings, ui_space, ui_text,
	ui_win_,
};

pub enum WindowType {
	WindowEmpty = 0,
	WindowHead = 1,
	WindowBody = 2,
	WindowNormal = 3,
}
pub enum MoveType {
	MoveExact = 0,
	MoveFaceUser = 1,
	MovePosOnly = 2,
	MoveNone = 3,
}

pub struct WindowContext(PhantomData<*const ()>);

pub fn window(
	_ctx: &DrawContext,
	window_title: &str,
	mut pose: &mut crate::pose::Pose,
	size: Vec2,
	window_type: WindowType,
	move_type: MoveType,
	mut content_closure: impl FnMut(&WindowContext),
) {
	let my_c_string = CString::new(window_title).unwrap();
	unsafe {
		stereokit_sys::ui_window_begin(
			my_c_string.as_ptr(),
			pose as *mut _ as *mut pose_t,
			vec2_from(size),
			window_type as ui_win_,
			move_type as ui_move_,
		)
	};

	content_closure(&WindowContext(PhantomData));

	unsafe {
		stereokit_sys::ui_window_end();
	}
}

impl WindowContext {
	pub fn sameline(&self) {
		unsafe { ui_sameline() }
	}
	pub fn space(&self, space: f32) {
		unsafe { ui_space(space) }
	}
	pub fn text(&self, text: &str, text_align: TextAlign) {
		let my_string = CString::new(text).unwrap();
		unsafe { ui_text(my_string.as_ptr(), text_align.bits()) }
	}
	pub fn label(&self, text: &str, use_padding: bool) {
		let my_string = CString::new(text).unwrap();
		unsafe { ui_label(my_string.as_ptr(), use_padding as bool32_t) }
	}
}
