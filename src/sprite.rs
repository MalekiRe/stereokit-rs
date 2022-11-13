use std::ptr::{NonNull, null_mut};
use stereokit_sys::{_model_t, _sprite_t};
use ustr::ustr;
use crate::lifecycle::StereoKitInstanceWrapper;
use crate::StereoKit;

pub struct Sprite {
    sk: StereoKitInstanceWrapper,
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
    pub fn from_file(sk: &StereoKit, file: &str, sprite_type: SpriteType) -> Self {
        Self {
            sk: sk.get_wrapper(),
            sprite: NonNull::new(unsafe {
                stereokit_sys::sprite_create_file(ustr(file).as_char_ptr(), sprite_type as u32, ustr("").as_char_ptr())
            }).unwrap()
        }
    }
}