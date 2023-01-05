pub trait StereoKitTime {
	fn time_elapsed(&self) -> f64 {
		unsafe { stereokit_sys::time_elapsed() }
	}
	fn time_elapsedf(&self) -> f32 {
		unsafe { stereokit_sys::time_elapsedf() }
	}
	fn time_getf_unscaled(&self) -> f32 {
		unsafe { stereokit_sys::time_getf_unscaled() }
	}
	fn time_elapsed_unscaled(&self) -> f64 {
		unsafe { stereokit_sys::time_elapsed_unscaled() }
	}
	fn time_elapsedf_unscaled(&self) -> f32 {
		unsafe { stereokit_sys::time_elapsedf_unscaled() }
	}
	fn time_get(&self) -> f64 {
		unsafe { stereokit_sys::time_get() }
	}
	fn time_getf(&self) -> f32 {
		unsafe { stereokit_sys::time_getf() }
	}
	fn time_get_unscaled(&self) -> f64 {
		unsafe { stereokit_sys::time_get_unscaled() }
	}
}

stereokit_trait_impl!(StereoKitTime);
