use crate::values::{vec3_from, vec3_to, MVec3};
use std::path::PathBuf;
use std::ptr::NonNull;
use stereokit_sys::{_sound_t, sound_inst_t};
use ustr::ustr;

pub struct Sound {
	pub(crate) sound: NonNull<_sound_t>,
}

pub trait SoundT {
	fn get_sound_ptr(&self) -> NonNull<_sound_t>;
	fn play_sound(&self, position: impl Into<MVec3>, volume: f32) -> SoundInstance {
		SoundInstance {
			sound_instance: unsafe {
				stereokit_sys::sound_play(
					self.get_sound_ptr().as_ptr(),
					vec3_from(position.into()),
					volume,
				)
			},
		}
	}
}

impl Sound {
	pub fn from_file(path: impl Into<PathBuf>) -> Option<Self> {
		let str = ustr(path.into().as_path().to_str().unwrap());
		Some(Self {
			sound: NonNull::new(unsafe { stereokit_sys::sound_create(str.as_char_ptr()) })?,
		})
	}
}

impl SoundT for Sound {
	fn get_sound_ptr(&self) -> NonNull<_sound_t> {
		self.sound
	}
}

pub struct SoundInstance {
	sound_instance: sound_inst_t,
}

impl SoundInstance {
	pub fn stop(self) {
		unsafe { stereokit_sys::sound_inst_stop(self.sound_instance) }
	}
	pub fn set_volume(&mut self, volume: f32) {
		unsafe { stereokit_sys::sound_inst_set_volume(self.sound_instance, volume) }
	}
	pub fn set_position(&self, position: impl Into<MVec3>) {
		unsafe {
			stereokit_sys::sound_inst_set_pos(self.sound_instance, vec3_from(position.into()))
		}
	}
	pub fn get_position(&self) -> MVec3 {
		unsafe { vec3_to(stereokit_sys::sound_inst_get_pos(self.sound_instance)) }
	}
}

pub struct SoundStream {
	pub(crate) sound_t: NonNull<_sound_t>,
}

impl SoundStream {
	pub fn create(buffer_duration: f32) -> Self {
		Self {
			sound_t: NonNull::new(unsafe { stereokit_sys::sound_create_stream(buffer_duration) })
				.unwrap(),
		}
	}
	pub fn write_samples(&self, in_arr_samples: &[f32]) {
		unsafe {
			stereokit_sys::sound_write_samples(
				self.sound_t.as_ptr(),
				in_arr_samples.as_ptr(),
				in_arr_samples.len() as u64,
			);
		}
	}
	pub fn read_samples(&self, out_arr_samples: &mut [f32]) -> u64 {
		unsafe {
			stereokit_sys::sound_read_samples(
				self.sound_t.as_ptr(),
				out_arr_samples.as_mut_ptr(),
				out_arr_samples.len() as u64,
			)
		}
	}
	pub fn unread_samples(&self) -> u64 {
		unsafe { stereokit_sys::sound_unread_samples(self.sound_t.as_ptr()) }
	}
}

impl SoundT for SoundStream {
	fn get_sound_ptr(&self) -> NonNull<_sound_t> {
		self.sound_t
	}
}
