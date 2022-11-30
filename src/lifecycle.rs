use crate::model::Model;
use crate::pose::Pose;
use derive_builder::Builder;
use num_enum::TryFromPrimitive;
use once_cell::unsync::OnceCell;
use std::any::Any;
use std::cell::{Ref, RefCell};
use std::ffi::{c_void, CString};
use std::fmt::Error;
use std::marker::PhantomData;
use std::os::unix::thread;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::rc::{Rc, Weak};
use std::sync::Mutex;
use std::{mem, ptr};

use stereokit_sys::{
	assets_releaseref_threadsafe, bool32_t, color32, depth_mode_, display_blend_, display_mode_,
	log_, material_t, model_t, pose_t, sk_settings_t,
};
use crate::render::StereoKitRender;

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum DisplayMode {
	MixedReality = 0,
	Flatscreen = 1,
	None = 2,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum DisplayBlend {
	None = 0,
	Opaque = 1,
	Additive = 2,
	Blend = 4,
	AnyTransparent = 6,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum DepthMode {
	Balanced = 0,
	D16 = 1,
	D32 = 2,
	Stencil = 3,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum LogFilter {
	None = 0,
	Diagnostic = 1,
	Inform = 2,
	Warning = 3,
	Error = 4,
}

thread_local! {
	static GLOBAL_STATE: RefCell<bool> = RefCell::new(false);
}

#[derive(Builder)]
#[builder(name = "Settings", pattern = "owned", setter(into), build_fn(skip))]
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
	disable_desktop_input_window: bool,
	disable_unfocused_sleep: bool,
}

impl Settings {
	pub fn init(self) -> Option<StereoKit> {
		if GLOBAL_STATE.with(|f| *f.borrow()) {
			return None;
		}
		let (vm_pointer, jobject_pointer) = (null_mut::<c_void>(), null_mut::<c_void>());
		#[cfg(target_os = "android")]
		let (vm_pointer, jobject_pointer) = {
			{
				let context = ndk_context::android_context();
				(context.vm(), context.context())
			}
		};
		let c_settings = sk_settings_t {
			app_name: CString::new(
				self.app_name
					.unwrap_or_else(|| "sk_app".to_owned())
					.as_str(),
			)
			.ok()?
			.into_raw(),
			assets_folder: CString::new(self.assets_folder.unwrap_or_default().as_str())
				.ok()?
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
			disable_desktop_input_window: self.disable_desktop_input_window.unwrap_or_default()
				as bool32_t,
			disable_unfocused_sleep: self.disable_unfocused_sleep.unwrap_or_default() as bool32_t,
			android_java_vm: vm_pointer,
			android_activity: jobject_pointer,
		};
		unsafe {
			if stereokit_sys::sk_init(c_settings) != 0 {
				GLOBAL_STATE.with(|f| *f.borrow_mut() = true);
				Some(StereoKit {
					ran: OnceCell::new(),
					lifetime_constraint: PhantomData,
				})
			} else {
				None
			}
		}
	}
}

pub struct StereoKit {
	ran: OnceCell<()>,
	lifetime_constraint: PhantomData<*const ()>,
}
pub struct DrawContext(PhantomData<*const ()>);

pub trait StereoKitContext {}

stereokit_trait_impl!(StereoKitContext);

type PanicPayload = Box<dyn Any + Send + 'static>;

/// SAFETY: payload_ptr must point to a value of type
/// `(&mut F, LST, GST, &mut Option<PanicPayload>)`.
/// It must also not be called synchronously with itself
/// or any other callback using the same parameters (due to &mut).
/// If `caught_panic` is written to, `F` and `LST` are
/// panic-poisoned, and the panic should likely be propagated.
unsafe extern "C" fn callback_trampoline<F, LST, GST>(payload_ptr: *mut c_void)
where
	F: FnMut(&mut LST, &mut GST),
{
	let payload =
		&mut *(payload_ptr as *mut (&mut F, &mut LST, &mut GST, &mut Option<PanicPayload>));
	let (closure, state, global_state, caught_panic) = payload;

	if caught_panic.is_some() {
		// we should consider the state poisoned and not run the callback
		return;
	}

	// the caller should ensure closure variables and state cannot be observed
	// after the panic without catching the panic,
	// which will in turn require them to be UnwindSafe
	let mut closure = AssertUnwindSafe(closure);
	let mut state = AssertUnwindSafe(state);
	// TODO: is global state always safe to be re-observed after a shutdown?
	let mut global_state = AssertUnwindSafe(global_state);

	let result = std::panic::catch_unwind(move || closure(*state, *global_state));
	if let Err(panic_payload) = result {
		caught_panic.replace(panic_payload);
		stereokit_sys::sk_quit();
	}
}

impl StereoKit {
	pub fn run(
		self,
		mut on_update: impl FnMut(&mut StereoKit, &DrawContext),
		mut on_close: impl FnMut(&mut StereoKit),
	) {
		self._run(
			&mut (),
			|(), (sk, dc)| on_update(*sk, dc),
			|_, (sk, _)| on_close(sk),
		);
	}
	pub fn run_stateful<ST>(
		self,
		state: &mut ST,
		mut on_update: impl FnMut(&mut ST, &mut StereoKit, &DrawContext),
		mut on_close: impl FnMut(&mut ST, &mut StereoKit),
	) {
		self._run(
			state,
			|st, (sk, dc)| on_update(st, *sk, dc),
			|st, (sk, _)| on_close(st, sk),
		);
	}

	fn _run<ST, U, S>(mut self, state: &mut ST, mut update: U, mut shutdown: S)
	where
		U: FnMut(&mut ST, &mut (&mut StereoKit, &DrawContext)),
		S: FnMut(&mut ST, &mut (&mut StereoKit, &DrawContext)),
	{
		let draw_context = DrawContext(PhantomData);

		// use one variable so shutdown doesn't run if update panics
		let mut caught_panic = Option::<PanicPayload>::None;

		let mut update_ref: (
			&mut U,
			&mut ST,
			&mut (&mut StereoKit, &DrawContext),
			&mut Option<PanicPayload>,
		) = (
			&mut update,
			state,
			&mut (&mut self, &draw_context),
			&mut caught_panic,
		);
		let update_raw = &mut update_ref
			as *mut (
				&mut U,
				&mut ST,
				&mut (&mut StereoKit, &DrawContext),
				&mut Option<PanicPayload>,
			) as *mut c_void;

		let mut shutdown_ref: (
			&mut S,
			&mut ST,
			&mut (&mut StereoKit, &DrawContext),
			&mut Option<PanicPayload>,
		) = (
			&mut shutdown,
			state,
			&mut (&mut self, &draw_context),
			&mut caught_panic,
		);
		let shutdown_raw = &mut shutdown_ref
			as *mut (
				&mut S,
				&mut ST,
				&mut (&mut StereoKit, &DrawContext),
				&mut Option<PanicPayload>,
			) as *mut c_void;

		if self.ran.set(()).is_err() {
			return;
		}

		unsafe {
			stereokit_sys::sk_run_data(
				Some(callback_trampoline::<U, ST, (&mut StereoKit, &DrawContext)>),
				update_raw,
				Some(callback_trampoline::<S, ST, (&mut StereoKit, &DrawContext)>),
				shutdown_raw,
			);
		}

		if let Some(panic_payload) = caught_panic {
			std::panic::resume_unwind(panic_payload);
		}
	}

	pub fn quit(&self) {
		unsafe { stereokit_sys::sk_quit() };
	}
}

impl Drop for StereoKit {
	fn drop(&mut self) {
		if self.ran.get().is_none() {
			unsafe { stereokit_sys::sk_shutdown() }
		};
	}
}
