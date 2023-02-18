#![allow(non_upper_case_globals)]

use crate::font::Font;
use crate::lifecycle::{StereoKitContext, StereoKitDraw};
use crate::values::{matrix_from, vec2_from, vec2_to, Color128, Color32, MMatrix, MVec2, MVec3, IntegerType};
use crate::StereoKit;
use bitflags::bitflags;
use bitflags_serde_shim::impl_serde_for_bitflags;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::rc::{Rc, Weak};
use stereokit_sys::{text_add_at, text_add_in, text_make_style, text_size, text_style_t};

bitflags! {
	pub struct TextAlign: u32 {
		const XLeft        = 1 << 0;
		const YTop         = 1 << 1;
		const XCenter      = 1 << 2;
		const YCenter      = 1 << 3;
		const XRight       = 1 << 4;
		const YBottom      = 1 << 5;
		const Center       = Self::XCenter.bits | Self::YCenter.bits;
		const CenterLeft   = Self::XLeft.bits   | Self::YCenter.bits;
		const CenterRight  = Self::XRight.bits  | Self::YCenter.bits;
		const TopCenter    = Self::XCenter.bits | Self::YTop.bits;
		const TopLeft      = Self::XLeft.bits   | Self::YTop.bits;
		const TopRight     = Self::XRight.bits  | Self::YTop.bits;
		const BottomCenter = Self::XCenter.bits | Self::YBottom.bits;
		const BottomLeft   = Self::XLeft.bits   | Self::YBottom.bits;
		const BottomRight  = Self::XRight.bits  | Self::YBottom.bits;
	}
}
impl_serde_for_bitflags!(TextAlign);

#[derive(Debug, Clone, Copy, TryFromPrimitive, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum TextFit {
	Wrap = 1,
	Clip = 2,
	Squeeze = 4,
	Exact = 8,
	Overflow = 16,
	//TODO remove maybe?
	WrapSqueeze = 4 | 1,
	WrapExact = 1 | 8,
}

#[derive(Clone)]
pub struct TextStyle {
	pub(crate) text_style: text_style_t,
}

impl TextStyle {
	pub fn new(
		_sk: &impl StereoKitContext,
		font: Font,
		character_height: f32,
		color_gamma: impl Into<Color128>,
	) -> TextStyle {
		TextStyle {
			text_style: unsafe {
				text_make_style(font.font.as_ptr(), character_height, color_gamma.into())
			},
		}
	}
	pub fn default(_sk: &impl StereoKitContext) -> TextStyle {
		TextStyle {
			text_style: 0 as text_style_t,
		}
	}
}

pub fn draw_at(
	_draw_ctx: &StereoKitDraw,
	text: impl AsRef<str>,
	transform: impl Into<MMatrix>,
	style: &TextStyle,
	position: TextAlign,
	align: TextAlign,
	offset: impl Into<MVec3>,
	vertex_tint_linear: impl Into<Color128>,
) {
	let text = ustr::ustr(text.as_ref());
	let offset = offset.into();
	unsafe {
		text_add_at(
			text.as_char_ptr(),
			&matrix_from(transform.into()),
			style.text_style,
			position.bits() as IntegerType,
			align.bits() as IntegerType,
			offset.x,
			offset.y,
			offset.z,
			vertex_tint_linear.into(),
		);
	}
}

pub fn draw_in(
	_draw_ctx: &StereoKitDraw,
	text: impl AsRef<str>,
	transform: impl Into<MMatrix>,
	size: impl Into<MVec2>,
	fit: TextFit,
	style: &TextStyle,
	position: TextAlign,
	align: TextAlign,
	offset: impl Into<MVec3>,
	vertex_tint_linear: impl Into<Color128>,
) -> f32 {
	let text = ustr::ustr(text.as_ref());
	let offset = offset.into();
	unsafe {
		text_add_in(
			text.as_char_ptr(),
			&matrix_from(transform.into()),
			vec2_from(size.into()),
			fit as IntegerType,
			style.text_style,
			position.bits() as IntegerType,
			align.bits() as IntegerType,
			offset.x,
			offset.y,
			offset.z,
			vertex_tint_linear.into(),
		)
	}
}

pub fn size(text: impl AsRef<str>, style: &TextStyle) -> MVec2 {
	let text = ustr::ustr(text.as_ref());
	unsafe { vec2_to(text_size(text.as_char_ptr(), style.text_style)) }
}
