use crate::lifecycle::StereoKitContext;
use crate::StereoKit;
use color_eyre::Report;
use color_eyre::Result;
use std::ptr::{null_mut, NonNull};
use stereokit_sys::{_model_t, _sprite_t};
use ustr::ustr;

pub struct Sprite {
	pub(crate) sprite: NonNull<_sprite_t>,
}
pub enum SpriteType {
	Atlased = 0,
	Single = 1,
}
impl Drop for Sprite {
	fn drop(&mut self) {
		unsafe {
			stereokit_sys::sprite_release(self.sprite.as_ptr());
		}
	}
}
impl Sprite {
	pub fn from_file(
		sk: &impl StereoKitContext,
		file: &str,
		sprite_type: SpriteType,
	) -> Result<Self> {
		Ok(Self {
			sprite: NonNull::new(unsafe {
				stereokit_sys::sprite_create_file(
					ustr(file).as_char_ptr(),
					sprite_type as u32,
					ustr("").as_char_ptr(),
				)
			})
			.ok_or(Report::msg(format!(
				"Unable to create sprite from file '{}'",
				file
			)))?,
		})
	}
}
