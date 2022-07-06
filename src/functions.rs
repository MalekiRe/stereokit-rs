use std::ffi::{CString, c_void};
use std::{mem, ptr};
use std::path::{Path, PathBuf};
use std::ptr::null;
use std::sync::Mutex;
use derive_builder::Builder;
use stereokit_sys::{assets_releaseref_threadsafe, bool32_t, color32, depth_mode_, display_blend_, display_mode_, log_, material_t, model_t, sk_settings_t};
use crate::enums::{DepthMode, DisplayBlend, DisplayMode, LogFilter};
use crate::model::Model;

pub type SKSettings = sk_settings_t;
// pub struct sk_settings_t {
// 	pub app_name: *const ::std::os::raw::c_char,
// 	pub assets_folder: *const ::std::os::raw::c_char,
// 	pub display_preference: display_mode_,
// 	pub blend_preference: display_blend_,
// 	pub no_flatscreen_fallback: bool32_t,
// 	pub depth_mode: depth_mode_,
// 	pub log_filter: log_,
// 	pub overlay_app: bool32_t,
// 	pub overlay_priority: u32,
// 	pub flatscreen_pos_x: i32,
// 	pub flatscreen_pos_y: i32,
// 	pub flatscreen_width: i32,
// 	pub flatscreen_height: i32,
// 	pub disable_flatscreen_mr_sim: bool32_t,
// 	pub disable_unfocused_sleep: bool32_t,
// 	pub android_java_vm: *mut ::std::os::raw::c_void,
// 	pub android_activity: *mut ::std::os::raw::c_void,
// }

#[derive(Builder)]
#[builder(name = "SKSettingBuilder", pattern = "owned")]
pub struct SKSettings2 {
	#[builder(default = "String::from(\"sk_app\")")]
	pub app_name: String,
	#[builder(default = "String::from(\"\")")]
	pub assets_folder: String,
	#[builder(default = "DisplayMode::MixedReality")]
	pub display_preference: DisplayMode,
	#[builder(default = "DisplayBlend::None")]
	pub blend_preference: DisplayBlend,
	#[builder(default = "false")]
	pub no_flatscreen_fallback: bool,
	#[builder(default = "DepthMode::Balanced")]
	pub depth_mode: DepthMode,
	#[builder(default = "LogFilter::Warning")]
	pub log_filter: LogFilter,
	#[builder(default = "false")]
	pub overlay_app: bool,
	#[builder(default = "1")]
	pub overlay_priority: u32,
	#[builder(default = "200")]
	pub flatscreen_pos_x: u32,
	#[builder(default = "200")]
	pub flatscreen_pos_y: u32,
	#[builder(default = "100")]
	pub flatscreen_width: u32,
	#[builder(default = "100")]
	pub flatscreen_height: u32,
	#[builder(default = "false")]
	pub disable_flatscreen_mr_sim: bool,
	#[builder(default = "false")]
	pub disable_unfocused_sleep: bool,
}

impl SKSettings2 {
	pub(crate) fn to_sk_settings(self) -> sk_settings_t {
		sk_settings_t {
			app_name: CString::new(self.app_name.as_str()).unwrap().into_raw(),
			assets_folder: CString::new(self.assets_folder.as_str()).unwrap().into_raw(),
			display_preference: self.display_preference as display_mode_,
			blend_preference: self.blend_preference as display_blend_,
			no_flatscreen_fallback: self.no_flatscreen_fallback as bool32_t,
			depth_mode: self.depth_mode as depth_mode_,
			log_filter: self.log_filter as log_,
			overlay_app: self.overlay_app as bool32_t,
			overlay_priority: self.overlay_priority,
			flatscreen_pos_x: self.flatscreen_pos_x as i32,
			flatscreen_pos_y: self.flatscreen_pos_y as i32,
			flatscreen_width: self.flatscreen_width as i32,
			flatscreen_height: self.flatscreen_height as i32,
			disable_flatscreen_mr_sim: self.disable_flatscreen_mr_sim as bool32_t,
			disable_unfocused_sleep: self.disable_unfocused_sleep as bool32_t,
			android_java_vm: ptr::null_mut(),
			android_activity: ptr::null_mut()
		}
	}
}

pub fn sk_init(settings: SKSettings2) -> bool {
	unsafe {
		if stereokit_sys::sk_init(settings.to_sk_settings()) != 0 {
			return true;
		}
		return false;
	}
}
pub fn sk_shutdown() {
	unsafe {
		stereokit_sys::sk_shutdown();
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
pub enum Asset {
	Model(Model),
	//Font(Font)
}
pub fn asset_releaseref(asset: Asset) {
	match asset {
		Asset::Model(mut model) => {
			let my_var = &mut model.model as *mut _ as *mut std::ffi::c_void;
			unsafe {
				assets_releaseref_threadsafe(my_var);
			}
		}
	}
}