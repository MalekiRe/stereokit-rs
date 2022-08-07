use crate::enums::{DepthMode, DisplayBlend, DisplayMode, LogFilter};
use crate::model::Model;
use derive_builder::Builder;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::ffi::{c_void, CString};
use std::path::{Path, PathBuf};
use std::ptr::null;
use std::sync::Mutex;
use std::{mem, ptr};
use stereokit_sys::{
    assets_releaseref_threadsafe, bool32_t, color32, depth_mode_, display_blend_, display_mode_,
    log_, material_t, model_t, sk_settings_t,
};

#[derive(Builder)]
#[builder(name = "SKSettings", pattern = "owned", setter(into), build_fn(skip))]
pub struct SKSettingsBuilt {
    app_name: String,
    assets_folder: String,
    display_preference: DisplayMode,
    blend_preference: DisplayBlend,
    no_flatscreen_fallback: bool,
    depth_mode: DepthMode,
    log_filter: LogFilter,
    overlay_app: bool,
    overlay_priority: u32,
    flatscreen_pos_x: u32,
    flatscreen_pos_y: u32,
    flatscreen_width: u32,
    flatscreen_height: u32,
    disable_flatscreen_mr_sim: bool,
    disable_unfocused_sleep: bool,
}

impl SKSettings {
    pub fn init(self) {
        let c_settings = sk_settings_t {
            app_name: CString::new(self.app_name.unwrap_or("sk_app".to_owned()).as_str())
                .unwrap()
                .into_raw(),
            assets_folder: CString::new(self.assets_folder.unwrap_or_default().as_str())
                .unwrap()
                .into_raw(),
            display_preference: self.display_preference.unwrap_or(DisplayMode::MixedReality)
                as display_mode_,
            blend_preference: self.blend_preference.unwrap_or(DisplayBlend::None) as display_blend_,
            no_flatscreen_fallback: self.no_flatscreen_fallback.unwrap_or_default() as bool32_t,
            depth_mode: self.depth_mode.unwrap_or(DepthMode::Balanced) as depth_mode_,
            log_filter: self.log_filter.unwrap_or(LogFilter::Warning) as log_,
            overlay_app: self.overlay_app.unwrap_or_default() as bool32_t,
            overlay_priority: self.overlay_priority.unwrap_or_default(),
            flatscreen_pos_x: self.flatscreen_pos_x.unwrap_or_default() as i32,
            flatscreen_pos_y: self.flatscreen_pos_y.unwrap_or_default() as i32,
            flatscreen_width: self.flatscreen_width.unwrap_or_default() as i32,
            flatscreen_height: self.flatscreen_height.unwrap_or_default() as i32,
            disable_flatscreen_mr_sim: self.disable_flatscreen_mr_sim.unwrap_or_default()
                as bool32_t,
            disable_unfocused_sleep: self.disable_unfocused_sleep.unwrap_or_default() as bool32_t,
            android_java_vm: ptr::null_mut(),
            android_activity: ptr::null_mut(),
        };
        unsafe { if stereokit_sys::sk_init(c_settings) == 0 { panic!("sk_init failed!") }}
    }
}

pub fn sk_quit() {
    unsafe { stereokit_sys::sk_quit() }
}
std::thread_local! {
    static SK_UPDATE_FN: RefCell<Option<Box<dyn FnMut()>>> = RefCell::from(None);
    static SK_SHUTDOWN_FN: RefCell<Option<Box<dyn FnMut()>>> = RefCell::from(None);
}

pub fn sk_run(on_update: impl FnMut() + 'static, on_close: impl FnMut() + 'static) {
    SK_UPDATE_FN.with(|f| *f.borrow_mut() = Some(Box::new(on_update)));
    SK_SHUTDOWN_FN.with(|f| *f.borrow_mut() = Some(Box::new(on_close)));
    unsafe {
        stereokit_sys::sk_run(Some(private_sk_run_func), Some(private_sk_close_func));
    }
}
extern "C" fn private_sk_run_func() {
    // let on_update_func: &mut Box<&mut dyn FnMut()> = unsafe { mem::transmute(context) };
    SK_UPDATE_FN.with(|f| {
        if let Some(update_fn) = f.borrow_mut().as_mut() {
            update_fn();
        }
    });
}
extern "C" fn private_sk_close_func() {
    // let on_update_func: &mut Box<&mut dyn FnMut()> = unsafe { mem::transmute(context) };
    SK_SHUTDOWN_FN.with(|f| {
        if let Some(shutdown_fn) = f.borrow_mut().as_mut() {
            shutdown_fn();
        }
    });
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
