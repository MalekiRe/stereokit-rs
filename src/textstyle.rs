use crate::font::Font;
use crate::values::{color128_from, Color128};
use crate::StereoKit;
use stereokit_sys::{text_make_style, text_style_t};

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
