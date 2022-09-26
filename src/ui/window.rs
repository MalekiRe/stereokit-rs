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
use ustr::ustr;
#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Component))]
pub enum WindowType {
	WindowEmpty = 0,
	WindowHead = 1,
	WindowBody = 2,
	WindowNormal = 3,
}
#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Component))]
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
	pose: &mut crate::pose::Pose,
	size: Vec2,
	window_type: WindowType,
	move_type: MoveType,
	content_closure: impl FnOnce(&WindowContext),
) {
	let window_title = ustr(window_title);
	unsafe {
		stereokit_sys::ui_window_begin(
			window_title.as_char_ptr(),
			std::mem::transmute(pose),
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
		let text = ustr(text);
		unsafe { ui_text(text.as_char_ptr(), text_align.bits()) }
	}
	pub fn label(&self, text: &str, use_padding: bool) {
		let text = ustr(text);
		unsafe { ui_label(text.as_char_ptr(), use_padding as bool32_t) }
	}
}
