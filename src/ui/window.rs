use crate::{
	lifecycle::DrawContext,
	text::TextAlign,
	values::{vec2_from, Vec2},
};
use num_enum::TryFromPrimitive;
use std::{ffi::CString, marker::PhantomData};
use stereokit_sys::{bool32_t, pose_t, text_align_, text_make_style, text_style_get_material, text_style_t, ui_btn_layout_, ui_button, ui_button_at, ui_button_img, ui_button_img_16, ui_button_img_at, ui_button_img_sz, ui_hslider, ui_label, ui_move_, ui_pop_text_style, ui_push_text_style, ui_sameline, ui_settings, ui_space, ui_text, ui_win_};
use ustr::ustr;
use crate::sprite::Sprite;
use crate::font::Font;
use crate::text::TextStyle;
use crate::values::{Vec3, vec3_from};

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Component))]
pub enum WindowType {
	WindowEmpty = 0,
	WindowHead = 1,
	WindowBody = 2,
	WindowNormal = 3,
}
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
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

pub fn try_window<Res, Er>(
	_ctx: &DrawContext,
	window_title: &str,
	pose: &mut crate::pose::Pose,
	size: Vec2,
	window_type: WindowType,
	move_type: MoveType,
	content_closure: impl FnOnce(&WindowContext) -> Result<Res, Er>,
) -> Result<Res, Er>{
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

	let result = content_closure(&WindowContext(PhantomData));

	unsafe {
		stereokit_sys::ui_window_end();
	}
	result
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
	pub fn button(&self, text: &str) -> bool {
		let text = ustr(text);
		unsafe {ui_button(text.as_char_ptr()) != 0}
	}
	pub fn button_image(&self, text: &str, sprite: &Sprite, layout: ButtonLayout) -> bool {
		unsafe {
			ui_button_img(ustr(text).as_char_ptr(), sprite.sprite.as_ptr(), layout as u32) != 0
		}
	}
	pub fn button_at(&self, text: &str, window_relative_pos: Vec3, size: Vec2) -> bool {
		unsafe {
			ui_button_at(ustr(text).as_char_ptr(), vec3_from(window_relative_pos), vec2_from(size)) != 0
		}
	}
	pub fn button_image_at(&self, text: &str, sprite: &Sprite, layout: ButtonLayout, window_relative_pos: Vec3, size: Vec2) -> bool {
		unsafe {
			ui_button_img_at(ustr(text).as_char_ptr(), sprite.sprite.as_ptr(), layout as u32, vec3_from(window_relative_pos), vec2_from(size)) != 0
		}
	}
	pub fn slider(&self, text: &str, val: &mut f32, min: f32, max: f32, step: f32, width: f32, confirm_method: ConfirmMethod) {
		unsafe {
			ui_hslider(ustr(text).as_char_ptr(), val as *mut f32, min, max, step, width,confirm_method as u32, 0);
		}
	}
	pub fn text_style(&self, text_style: TextStyle, content_closure: impl FnOnce(&WindowContext)) {
		unsafe {
			ui_push_text_style(text_style.text_style)
		}
		content_closure(&self);
		unsafe {
			ui_pop_text_style();
		}
	}
	pub fn try_text_style<Res, Er>(&self, text_style: TextStyle, content_closure: impl FnOnce(&WindowContext) -> Result<Res, Er>) -> Result<Res, Er> {
		unsafe {
			ui_push_text_style(text_style.text_style)
		}
		let result = content_closure(&self);
		unsafe {
			ui_pop_text_style();
		}
		result
	}
}
pub enum ConfirmMethod {
	Push = 0,
	Pinch = 1,
	VariablePinch = 2,
}
pub enum ButtonLayout {
	Left = 0,
	Right = 1,
	Center = 2,
	CenterNoText = 3,
}