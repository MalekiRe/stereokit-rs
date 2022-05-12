use std::ffi::{CString, c_void};
use std::mem;
use std::sync::Mutex;
use stereokit_sys::{color32, material_t, sk_settings_t};

pub fn sk_init(settings: sk_settings_t) -> bool {
	unsafe {
		if stereokit_sys::sk_init(settings) != 0 {
			return true;
		}
		return false;
	}
}
pub fn sk_run_data(on_update: &mut Box<&mut dyn FnMut()>, on_close: &mut Box<&mut dyn FnMut()>) {
	let on_update_c_void: *mut c_void = on_update as *mut _ as *mut c_void;
	let on_close_c_void: *mut c_void = on_close as *mut _ as *mut c_void;
	unsafe { stereokit_sys::sk_run_data(Some(private_sk_run_func), on_update_c_void, Some(private_sk_run_func), on_close_c_void) }
}
extern "C" fn private_sk_run_func(context: *mut c_void) {
	let on_update_func: &mut Box<&mut dyn FnMut()> = unsafe {mem::transmute(context)};
	on_update_func()
}