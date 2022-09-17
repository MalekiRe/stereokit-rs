use crate::StereoKit;

impl StereoKit {
	pub fn time_elapsed(&self) -> f64 {
		unsafe { stereokit_sys::time_elapsed() }
	}
	pub fn time_elapsedf(&self) -> f32 {
		unsafe { stereokit_sys::time_elapsedf() }
	}
	pub fn time_getf_unscaled(&self) -> f32 {
		unsafe { stereokit_sys::time_getf_unscaled() }
	}
	pub fn time_elapsed_unscaled(&self) -> f64 {
		unsafe { stereokit_sys::time_elapsed_unscaled() }
	}
	pub fn time_elapsedf_unscaled(&self) -> f32 {
		unsafe { stereokit_sys::time_elapsedf_unscaled() }
	}
	pub fn time_get(&self) -> f64 {
		unsafe { stereokit_sys::time_get() }
	}
	pub fn time_getf(&self) -> f32 {
		unsafe { stereokit_sys::time_getf() }
	}
	pub fn time_get_unscaled(&self) -> f64 {
		unsafe { stereokit_sys::time_get_unscaled() }
	}
}
