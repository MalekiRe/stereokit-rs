#![allow(non_upper_case_globals)]

use crate::font::Font;
use crate::values::{color128_from, Color128};
use crate::StereoKit;
use bitflags::bitflags;
use stereokit_sys::{text_make_style, text_style_t};

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

#[derive(Clone, Copy)]
pub struct TextStyle<'a> {
	sk: &'a StereoKit<'a>,
	pub(crate) text_style: text_style_t,
}

impl<'a> TextStyle<'a> {
	pub fn make_style(
		sk: &'a StereoKit,
		font: Font,
		character_height: f32,
		color_gamma: Color128,
	) -> Self {
		TextStyle {
			sk,
			text_style: unsafe {
				text_make_style(font.font, character_height, color128_from(color_gamma))
			},
		}
	}
}
