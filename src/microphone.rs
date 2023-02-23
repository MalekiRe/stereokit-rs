use crate::sound::SoundStream;
use color_eyre::Report;
use color_eyre::Result;
use std::ffi::CStr;
use std::ptr::NonNull;
pub struct Microphone {
	name: String,
}

impl Microphone {
	pub fn device_count() -> u32 {
		unsafe { stereokit_sys::mic_device_count() as u32 }
	}
	pub fn device_name(device_num: u32) -> String {
		unsafe { CStr::from_ptr(stereokit_sys::mic_device_name(device_num as i32)) }
			.to_str()
			.unwrap()
			.to_string()
	}
	pub fn _start(device_name: &str) -> bool {
		let device_name = ustr::ustr(device_name);
		unsafe { stereokit_sys::mic_start(device_name.as_char_ptr()) != 0 }
	}
	pub fn _stop() {
		unsafe {
			stereokit_sys::mic_stop();
		}
	}
	pub fn _get_stream() -> Result<SoundStream> {
		Ok(SoundStream {
			sound_t: NonNull::new(unsafe { stereokit_sys::mic_get_stream() })
				.ok_or(Report::msg("microphone isn't on"))?,
		})
	}
	pub fn _is_recording() -> bool {
		unsafe { stereokit_sys::mic_is_recording() != 0 }
	}
	pub fn new(device_num: u32) -> Self {
		Self {
			name: Self::device_name(device_num),
		}
	}
	pub fn start(&self) -> bool {
		Self::_start(self.name.as_str())
	}
	pub fn stop(&self) {
		Self::_stop();
	}
	pub fn get_stream(&self) -> Result<SoundStream> {
		Self::_get_stream()
	}
	pub fn is_recording(&self) -> bool {
		Self::_is_recording()
	}
	pub fn get_name(&self) -> &str {
		self.name.as_str()
	}
}
