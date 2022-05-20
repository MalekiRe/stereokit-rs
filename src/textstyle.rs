use stereokit_sys::{text_make_style, text_style_t};
use crate::font::Font;
use crate::values::{Color128, color128_from};

pub struct TextStyle {
	pub(crate)text_style: text_style_t
}

impl TextStyle {
	pub fn make_style(font: Font, character_height: f32, color_gamma: Color128) -> Self {
		TextStyle{
			text_style: unsafe {text_make_style(
				font.font,
				character_height,
				color128_from(color_gamma)
			)}
		}
	}
}