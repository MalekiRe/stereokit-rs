use std::path::PathBuf;
use std::ptr::NonNull;
use stereokit_sys::{_sound_t, sound_inst_t, sound_t};
use ustr::ustr;
use crate::values::{MVec3, vec3_from};

pub struct Sound {
    sound: NonNull<_sound_t>,
}

impl Sound {
    pub fn from_file(path: impl Into<PathBuf>) -> Option<Self> {
        let str = ustr(path.into().as_path().to_str().unwrap());
        Some(Self {
            sound: NonNull::new(unsafe {
            stereokit_sys::sound_create(str.as_char_ptr())
            })?
        })
    }
    pub fn play_sound(&self, position: impl Into<MVec3>, volume: f32) -> SoundInstance {
        SoundInstance{
            sound_instance: unsafe {
                stereokit_sys::sound_play(self.sound.as_ptr(), vec3_from(position.into()), volume)
            }
        }
    }
}

pub struct SoundInstance {
    sound_instance: sound_inst_t
}
impl SoundInstance {
    pub fn stop(self) {
        unsafe {
            stereokit_sys::sound_inst_stop(self.sound_instance)
        }
    }
    pub fn set_volume(&mut self, volume: f32) {
        unsafe {
            stereokit_sys::sound_inst_set_volume(self.sound_instance, volume)
        }
    }
}