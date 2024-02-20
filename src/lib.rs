#![doc = include_str!("../README.md")]
pub mod named_colors;
#[cfg(test)]
mod tests;

#[cfg(target_os = "windows")]
type IntegerType = i32;
#[cfg(not(target_os = "windows"))]
type IntegerType = u32;

#[cfg(feature = "bevy_ecs")]
use bevy_ecs::prelude::ReflectComponent;
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::FromReflect;
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::{DynamicInfo, Reflect, ReflectMut, ReflectOwned, ReflectRef, TypeInfo, ValueInfo};

use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sys::origin_mode_;
use std::any::Any;
use std::collections::HashSet;
use std::ffi::{c_void, CStr, CString};
use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut, slice_from_raw_parts_mut, NonNull};
use stereokit_sys::{_font_t, _gradient_t, _material_buffer_t, _material_t, _mesh_t, _model_t, _shader_t, _solid_t, _sound_t, _sprite_t, _tex_t, anim_mode_, app_focus_, bool32_t, bounds_t, controller_t, cull_, depth_mode_, depth_test_, device_tracking_, display_, display_blend_, display_mode_, display_type_, fov_info_t, gradient_key_t, hand_joint_t, hand_t, handed_, key_, line_point_t, log_, log_colors_, mesh_t, mouse_t, openxr_handle_t, plane_t, pointer_t, pose_t, projection_, quat, ray_t, rect_t, render_clear_, sh_light_t, sk_init, sk_settings_t, sound_inst_t, sphere_t, spherical_harmonics_t, sprite_type_, system_info_t, tex_address_, tex_format_, tex_sample_, text_align_, text_fit_, track_state_, transparency_, ui_color_, ui_cut_, ui_move_, ui_win_, ui_btn_layout_, vert_t, world_refresh_};
use thiserror::Error;

pub use stereokit_sys as sys;

pub struct SkDraw(PhantomData<*const ()>);
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Resource))]
pub struct Sk(PhantomData<()>);
pub struct SkSingle(pub(crate) PhantomData<*const ()>);

impl StereoKitSingleThread for SkDraw {}
impl StereoKitMultiThread for SkDraw {}
impl StereoKitDraw for SkDraw {}
impl StereoKitMultiThread for Sk {}
impl StereoKitMultiThread for SkSingle {}
impl StereoKitSingleThread for SkSingle {}

impl SkSingle {
	pub fn multithreaded(&self) -> Sk {
		Sk(PhantomData)
	}
	/// only use if you know what you are doing
	pub unsafe fn create_unsafe() -> SkSingle {
		SkSingle(PhantomData)
	}
}

impl Sk {
	/// only use if you know what you are doing
	pub unsafe fn create_unsafe() -> Sk {
		Sk(PhantomData)
	}
}

impl SkDraw {
	pub fn multithreaded(&self) -> Sk {
		Sk(PhantomData)
	}
	/// only use if you know what you are doing
	pub unsafe fn create_unsafe() -> Self {
		SkDraw(PhantomData)
	}
}

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

// static mut GLOBAL_THING: Option<Box<dyn FnMut(&CSkDraw)>> = None;
// static mut wait_for_me: bool = false;
//
// extern "C" fn private_sk_step_func() {
//     unsafe {
//         GLOBAL_THING.as_mut().unwrap()(&CSkDraw(PhantomData));
//         wait_for_me = false;
//     }
// }

impl SkSingle {
	// pub fn step(&mut self, mut on_step: impl FnMut(&CSkDraw) + 'static) {
	//     unsafe {
	//         while wait_for_me {}
	//         GLOBAL_THING.replace(Box::new(on_step));
	//         wait_for_me = true;
	//         stereokit_sys::sk_step(Some(private_sk_step_func));
	//     }
	// }
	pub fn run(self, mut on_update: impl FnMut(&SkDraw), mut on_close: impl FnMut(&mut SkSingle)) {
		self._run(
			&mut (),
			|(), (_sk, dc)| on_update(dc),
			|_, (sk, _)| on_close(sk),
		);
	}
	pub fn run_stateful<ST>(
		self,
		state: &mut ST,
		mut on_update: impl FnMut(&mut ST, &mut SkSingle, &SkDraw),
		mut on_close: impl FnMut(&mut ST, &mut SkSingle),
	) {
		self._run(
			state,
			|st, (sk, dc)| on_update(st, *sk, dc),
			|st, (sk, _)| on_close(st, sk),
		);
	}

	fn _run<ST, U, S>(mut self, state: &mut ST, mut update: U, mut shutdown: S)
	where
		U: FnMut(&mut ST, &mut (&mut SkSingle, &SkDraw)),
		S: FnMut(&mut ST, &mut (&mut SkSingle, &SkDraw)),
	{
		let draw_context = SkDraw(PhantomData);

		// use one variable so shutdown doesn't run if update panics
		let mut caught_panic = Option::<PanicPayload>::None;

		let mut update_ref: (
			&mut U,
			&mut ST,
			&mut (&mut SkSingle, &SkDraw),
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
				&mut (&mut SkSingle, &SkDraw),
				&mut Option<PanicPayload>,
			) as *mut c_void;

		let mut shutdown_ref: (
			&mut S,
			&mut ST,
			&mut (&mut SkSingle, &SkDraw),
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
				&mut (&mut SkSingle, &SkDraw),
				&mut Option<PanicPayload>,
			) as *mut c_void;

		// if self.ran.set(()).is_err() {
		//     return;
		// }

		unsafe {
			stereokit_sys::sk_run_data(
				Some(callback_trampoline::<U, ST, (&mut SkSingle, &SkDraw)>),
				update_raw,
				Some(callback_trampoline::<S, ST, (&mut SkSingle, &SkDraw)>),
				shutdown_raw,
			);
		}

		if let Some(panic_payload) = caught_panic {
			std::panic::resume_unwind(panic_payload);
		}
	}
}

pub type SkResult<T> = Result<T, StereoKitError>;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum StereoKitError {
	#[error("unable to create model from file path {0}")]
	ModelFile(String),
	#[error("unable to find model with id {0}")]
	ModelFind(String),
	#[error("failed to create model {0} from memory for reason {1}")]
	ModelFromMem(String, String),
	#[error("failed to create model {0} from file for reason {1}")]
	ModelFromFile(PathBuf, String),
	#[error("failed to find mesh {0}")]
	MeshFind(String),
	#[error("failed to convert to CString {0} in mesh_find")]
	MeshCString(String),
	#[error("failed to convert to CString {0} in tex_find")]
	TexCString(String),
	#[error("failed to find tex {0}")]
	TexFind(String),
	#[error("failed to create a tex from raw memory")]
	TexMemory,
	#[error("failed to create a tex from file {0} for reason {1}")]
	TexFile(PathBuf, String),
	#[error("failed to find font {0} for reason {1}")]
	FontFind(String, String),
	#[error("failed to create font from file {0} for reason {1}")]
	FontFile(PathBuf, String),
	#[error("failed to find shader {0} for reason {1}")]
	ShaderFind(String, String),
	#[error("failed to create shader from file {0} for reason {1}")]
	ShaderFile(PathBuf, String),
	#[error("failed to create shader from raw memory")]
	ShaderMem,
	#[error("failed to find material {0} for reason {1}")]
	MaterialFind(String, String),
	#[error("failed to create sprite from texture")]
	SpriteCreate,
	#[error("failed to create sprite from file {0}")]
	SpriteFile(String),
	#[error("failed to find sprite {0} for reason {1}")]
	SpriteFind(String, String),
	#[error("failed to find sound {0}")]
	SoundFind(String),
	#[error("failed to create sound from file {0}")]
	SoundCreate(PathBuf),
	#[error("failed to init stereokit with settings {0}")]
	SkInit(Settings),
}

pub type Color32 = stereokit_sys::color32;
pub type Color128 = stereokit_sys::color128;
pub type ModelNodeId = i32;

/// Specifies a type of display mode StereoKit uses, like Mixed Reality headset display vs. a PC display, or even just rendering to an offscreen surface, or not rendering at all!
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum DisplayMode {
	/// Creates an OpenXR instance, and drives display/input through that.
	MixedReality = 0,
	/// Creates a flat, Win32 window, and simulates some MR functionality. Great for debugging.
	Flatscreen = 1,
	/// Not tested yet, but this is meant to run StereoKit without rendering to any display at all. This would allow for rendering to textures, running a server that can do MR related tasks, etc.
	None = 2,
}
impl From<display_mode_> for DisplayMode {
	fn from(value: display_mode_) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}

/// TODO: remove this in v0.4 This describes the type of display tech used on a Mixed Reality device. This will be replaced by DisplayBlend in v0.4.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum Display {
	/// 	Default value, when using this as a search type, it will fall back to default behavior which defers to platform preference.
	None = 0,
	/// This display is opaque, with no view into the real world! This is equivalent to a VR headset, or a PC screen.
	Opaque = 1,
	/// This display is transparent, and adds light on top of the real world. This is equivalent to a HoloLens type of device.
	Additive = 2,
	/// This is a physically opaque display, but with a camera passthrough displaying the world behind it anyhow. This would be like a Varjo XR-1, or phone-camera based AR.
	BlendPassthrough = 4,
	/// This matches either transparent display type! Additive or Blend. For use when you just want to see the world behind your application.
	AnyTransparent = 6,
}
impl From<display_> for Display {
	fn from(value: display_) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}

/// This describes the way the display’s content blends with whatever is behind it. VR headsets are normally Opaque, but some VR headsets provide passthrough video, and can support Opaque as well as Blend, like the Varjo. Transparent AR displays like the HoloLens would be Additive.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum DisplayBlend {
	/// Default value, when using this as a search type, it will fall back to default behavior which defers to platform preference.
	None = 0,
	/// This display is opaque, with no view into the real world! This is equivalent to a VR headset, or a PC screen.
	Opaque = 1,
	/// This display is transparent, and adds light on top of the real world. This is equivalent to a HoloLens type of device.
	Additive = 2,
	/// This is a physically opaque display, but with a camera passthrough displaying the world behind it anyhow. This would be like a Varjo XR-1, or phone-camera based AR.
	Blend = 4,
	/// This matches either transparent display type! Additive or Blend. For use when you just want to see the world behind your application.
	AnyTransparent = 6,
}
impl From<display_blend_> for DisplayBlend {
	fn from(value: display_blend_) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<display_blend_> for DisplayBlend {
	fn into(self) -> display_blend_ {
		unsafe { std::mem::transmute(self) }
	}
}

/// The App initial reference point
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum OriginMode {
	/// Default value 
	Local = 0,
	/// Floor
	Floor = 1,
	/// Stage
	Stage = 2,
}

/// This is used to determine what kind of depth buffer StereoKit uses!
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum DepthMode {
	/// Default mode, uses 16 bit on mobile devices like HoloLens and Quest, and 32 bit on higher powered platforms like PC. If you need a far view distance even on mobile devices, prefer D32 or Stencil instead.
	Balanced = 0,
	/// 16 bit depth buffer, this is fast and recommended for devices like the HoloLens. This is especially important for fast depth based reprojection. Far view distances will suffer here though, so keep your clipping far plane as close as possible.
	D16 = 1,
	/// 32 bit depth buffer, should look great at any distance! If you must have the best, then this is the best. If you’re interested in this one, Stencil may also be plenty for you, as 24 bit depth is also pretty peachy.
	D32 = 2,
	/// 24 bit depth buffer with 8 bits of stencil data. 24 bits is generally plenty for a depth buffer, so using the rest for stencil can open up some nice options! StereoKit has limited stencil support right now though (v0.3).
	Stencil = 3,
}
impl From<depth_mode_> for DepthMode {
	fn from(value: depth_mode_) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}

/// Severity of a log item.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum LogLevel {
	None = 0,
	/// This is for diagnostic information, where you need to know details about what -exactly- is going on in the system. This info doesn’t surface by default.
	Diagnostic = 1,
	/// This is non-critical information, just to let you know what’s going on.
	Inform = 2,
	/// Something bad has happened, but it’s still within the realm of what’s expected.
	Warning = 3,
	/// Danger Will Robinson! Something really bad just happened and needs fixing!
	Error = 4,
}
impl From<log_> for LogLevel {
	fn from(value: log_) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}

bitflags::bitflags! {
/// When rendering content, you can filter what you’re rendering by the RenderLayer that they’re on. This allows you to draw items that are visible in one render, but not another. For example, you may wish to draw a player’s avatar in a ‘mirror’ rendertarget, but not in the primary display. See Renderer.LayerFilter for configuring what the primary display renders.
	#[derive(Serialize, Deserialize)]
	#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
	#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::prelude::Reflect, bevy_reflect::prelude::FromReflect))]
	#[cfg_attr(feature = "bevy_reflect", reflect(Component))]
	pub struct RenderLayer: u32 {
		/// The default render layer. All Draw use this layer unless otherwise specified.
		const LAYER0 = 1 << 0;
		/// Render layer 1.
		const LAYER1 = 1 << 1;
		/// Render layer 2.
		const LAYER2 = 1 << 2;
		/// Render layer 3.
		const LAYER3 = 1 << 3;
		/// Render layer 4.
		const LAYER4 = 1 << 4;
		/// Render layer 5.
		const LAYER5 = 1 << 5;
		/// Render layer 6.
		const LAYER6 = 1 << 6;
		/// Render layer 7.
		const LAYER7 = 1 << 7;
		/// Render layer 8.
		const LAYER8 = 1 << 8;
		/// Render layer 9.
		const LAYER9 = 1 << 9;
		/// The default VFX layer, StereoKit draws some non-standard mesh content using this flag, such as lines.
		const LAYER_VFX = 10;
		/// This is a flag that specifies all possible layers. If you want to render all layers, then this is the layer filter you would use. This is the default for render filtering.
		const LAYER_ALL = 0xFFFF;
		/// This is a combination of all layers that are not the VFX layer.
		const LAYER_ALL_REGULAR = Self::LAYER0.bits | Self::LAYER1.bits | Self::LAYER2.bits | Self::LAYER3.bits | Self::LAYER4.bits | Self::LAYER5.bits | Self::LAYER6.bits | Self::LAYER7.bits | Self::LAYER8.bits | Self::LAYER9.bits;
	}
}

impl Default for RenderLayer {
	fn default() -> Self {
		RenderLayer::LAYER_ALL
	}
}
/// For performance sensitive areas, or places dealing with large chunks of memory, it can be faster to get a reference to that memory rather than copying it! However, if this isn’t explicitly stated, it isn’t necessarily clear what’s happening. So this enum allows us to visibly specify what type of memory reference is occurring.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum Memory {
	/// The chunk of memory involved here is a reference that is still managed or used by StereoKit! You should not free it, and be extremely cautious about modifying it.
	Reference = 0,
	/// This memory is now yours and you must free it yourself! Memory has been allocated, and the data has been copied over to it. Pricey! But safe.
	Copy = 1,
}

/// StereoKit initialization settings!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
	/// Name of the application, this shows up an the top of the Win32 window, and is submitted to OpenXR. OpenXR caps this at 128 characters.
	pub app_name: String,
	/// Where to look for assets when loading files! Final path will look like ‘\[assetsFolder\]/\[file\]’, so a trailing ‘/’ is unnecessary.
	pub assets_folder: PathBuf,
	/// Which display type should we try to load? Default is DisplayMode.MixedReality.
	pub display_preference: DisplayMode,
	///If the preferred display fails, should we avoid falling back to flatscreen and just crash out? Default is false.
	pub no_flatscreen_fallback: bool,
	/// What type of background blend mode do we prefer for this application? Are you trying to build an Opaque/Immersive/VR app, or would you like the display to be AnyTransparent, so the world will show up behind your content, if that’s an option? Note that this is a preference only, and if it’s not available on this device, the app will fall back to the runtime’s preference instead! By default, (DisplayBlend.None) this uses the runtime’s preference.
	pub blend_preference: DisplayBlend,
	/// What kind of depth buffer should StereoKit use? A fast one, a detailed one, one that uses stencils? By default, StereoKit uses a balanced mix depending on platform, prioritizing speed but opening up when there’s headroom.
	pub depth_mode: DepthMode,
	/// The default log filtering level. This can be changed at runtime, but this allows you to set the log filter before Initialization occurs, so you can choose to get information from that. Default is LogLevel.Info.
	pub log_filter: LogLevel,
	/// If the runtime supports it, should this application run as an overlay above existing applications? Check SK.System.overlayApp after initialization to see if the runtime could comply with this flag. This will always force StereoKit to work in a blend compositing mode.
	pub overlay_app: bool,
	/// For overlay applications, this is the order in which apps should be composited together. 0 means first, bottom of the stack, and uint.MaxValue is last, on top of the stack.
	pub overlay_priority: u32,
	/// If using Runtime.Flatscreen, the pixel position of the window on the screen.
	pub flatscreen_pos_x: i32,
	/// If using Runtime.Flatscreen, the pixel position of the window on the screen.
	pub flatscreen_pos_y: i32,
	/// If using Runtime.Flatscreen, the pixel size of the window on the screen.
	pub flatscreen_width: i32,
	/// If using Runtime.Flatscreen, the pixel size of the window on the screen.
	pub flatscreen_height: i32,
	/// By default, StereoKit will simulate Mixed Reality input so developers can test MR spaces without being in a headset. If You don’t want this, you can disable it with this setting!
	pub disable_flatscreen_mr_sim: bool,
	pub disable_desktop_input_window: bool,
	/// By default, StereoKit will slow down when the application is out of focus. This is useful for saving processing power while the app is out-of-focus, but may not always be desired. In particular, running multiple copies of a SK app for testing networking code may benefit from this setting.
	pub disable_unfocused_sleep: bool,
	pub render_scaling: f32,
	pub origin: OriginMode,
}
impl Default for Settings {
	fn default() -> Self {
		Self {
			app_name: "StereoKit".to_string(),
			assets_folder: PathBuf::from(""),
			display_preference: DisplayMode::MixedReality,
			no_flatscreen_fallback: false,
			blend_preference: DisplayBlend::None,
			depth_mode: DepthMode::Balanced,
			log_filter: LogLevel::Warning,
			overlay_app: false,
			overlay_priority: 0,
			flatscreen_pos_x: 0,
			flatscreen_pos_y: 0,
			flatscreen_width: 0,
			flatscreen_height: 0,
			disable_flatscreen_mr_sim: false,
			disable_desktop_input_window: false,
			disable_unfocused_sleep: false,
			render_scaling: 1.0,
			origin : OriginMode::Local,
		}
	}
}
impl fmt::Display for Settings {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}
impl From<sk_settings_t> for Settings {
	fn from(value: sk_settings_t) -> Self {
		match value {
			sk_settings_t {
				app_name,
				assets_folder,
				display_preference,
				blend_preference,
				no_flatscreen_fallback,
				depth_mode,
				log_filter,
				overlay_app,
				overlay_priority,
				flatscreen_pos_x,
				flatscreen_pos_y,
				flatscreen_width,
				flatscreen_height,
				disable_flatscreen_mr_sim,
				disable_desktop_input_window,
				disable_unfocused_sleep,
				render_scaling,
				render_multisample: _,
				origin,
				android_java_vm: _,
				android_activity: _,
			} => unsafe {
				Self {
					app_name: CStr::from_ptr(app_name).to_str().unwrap().to_string(),
					assets_folder: CStr::from_ptr(assets_folder)
						.to_str()
						.unwrap()
						.to_string()
						.parse()
						.unwrap(),
					display_preference: std::mem::transmute(display_preference),
					no_flatscreen_fallback: no_flatscreen_fallback != 0,
					blend_preference: std::mem::transmute(blend_preference),
					depth_mode: std::mem::transmute(depth_mode),
					log_filter: std::mem::transmute(log_filter),
					overlay_app: overlay_app != 0,
					overlay_priority,
					flatscreen_pos_x,
					flatscreen_pos_y,
					flatscreen_width,
					flatscreen_height,
					disable_flatscreen_mr_sim: disable_flatscreen_mr_sim != 0,
					disable_desktop_input_window: disable_desktop_input_window != 0,
					disable_unfocused_sleep: disable_unfocused_sleep != 0,
					render_scaling,
					origin:std::mem::transmute(origin),
				}
			},
		}
	}
}
impl Into<sk_settings_t> for Settings {
	fn into(self) -> sk_settings_t {
		match self {
			Settings {
				app_name,
				assets_folder,
				display_preference,
				no_flatscreen_fallback,
				blend_preference,
				depth_mode,
				log_filter,
				overlay_app,
				overlay_priority,
				flatscreen_pos_x,
				flatscreen_pos_y,
				flatscreen_width,
				flatscreen_height,
				disable_flatscreen_mr_sim,
				disable_desktop_input_window,
				disable_unfocused_sleep,
				render_scaling,
				origin,
			} => {
				let app_name = CString::new(app_name).unwrap();
				let assets_folder = CString::new(assets_folder.to_str().unwrap()).unwrap();
				let s = sk_settings_t {
					app_name: app_name.into_raw(),
					assets_folder: assets_folder.into_raw(),
					display_preference: display_preference as display_mode_,
					blend_preference: blend_preference as display_blend_,
					no_flatscreen_fallback: no_flatscreen_fallback as bool32_t,
					depth_mode: depth_mode as depth_mode_,
					log_filter: log_filter as log_,
					overlay_app: overlay_app as bool32_t,
					overlay_priority,
					flatscreen_pos_x,
					flatscreen_pos_y,
					flatscreen_width,
					flatscreen_height,
					disable_flatscreen_mr_sim: disable_flatscreen_mr_sim as bool32_t,
					disable_desktop_input_window: disable_desktop_input_window as bool32_t,
					disable_unfocused_sleep: disable_unfocused_sleep as bool32_t,
					render_scaling,
					render_multisample: 1,
        			origin: origin as origin_mode_,
					android_java_vm: null_mut(),
					android_activity: null_mut(),
				};
				//Box::leak(Box::new(app_name));
				//Box::leak(Box::new(assets_folder));
				s
			}
		}
	}
}

impl Settings {
	pub fn init(self) -> SkResult<SkSingle> {
		let (vm_pointer, jobject_pointer) = (null_mut::<c_void>(), null_mut::<c_void>());
		#[cfg(target_os = "android")]
		let (vm_pointer, jobject_pointer) = {
			{
				let context = ndk_context::android_context();
				(context.vm(), context.context())
			}
		};
		let mut settings: sk_settings_t = self.clone().into();
		settings.android_java_vm = vm_pointer;
		settings.android_activity = jobject_pointer;
		match unsafe {
			println!("before init");
			let val = sk_init(settings) != 0;
			println!("after init");
			val
		} {
			true => Ok(SkSingle(std::marker::PhantomData)),
			false => Err(StereoKitError::SkInit(self)),
		}
	}
}
/// Use this to help construct your settings!
#[derive(Default, Debug)]
pub struct SettingsBuilder {
	settings: Settings,
}
impl SettingsBuilder {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn app_name(&mut self, app_name: impl AsRef<str>) -> &mut Self {
		self.settings.app_name = app_name.as_ref().to_string();
		self
	}
	pub fn assets_folder(&mut self, assets_folder: impl AsRef<Path>) -> &mut Self {
		self.settings.assets_folder = assets_folder.as_ref().to_path_buf();
		self
	}
	pub fn display_preference(&mut self, display_preference: DisplayMode) -> &mut Self {
		self.settings.display_preference = display_preference;
		self
	}
	pub fn blend_preference(&mut self, blend_preference: DisplayBlend) -> &mut Self {
		self.settings.blend_preference = blend_preference;
		self
	}
	pub fn no_flatscreen_fallback(&mut self, no_flatscreen_fallback: bool) -> &mut Self {
		self.settings.no_flatscreen_fallback = no_flatscreen_fallback;
		self
	}
	pub fn log_filter(&mut self, log_filter: LogLevel) -> &mut Self {
		self.settings.log_filter = log_filter;
		self
	}
	pub fn overlay_app(&mut self, overlay_app: bool) -> &mut Self {
		self.settings.overlay_app = overlay_app;
		self
	}
	pub fn overlay_priority(&mut self, overlay_priority: u32) -> &mut Self {
		self.settings.overlay_priority = overlay_priority;
		self
	}
	pub fn flatscreen_pos_x(&mut self, flatscreen_pos_x: i32) -> &mut Self {
		self.settings.flatscreen_pos_x = flatscreen_pos_x;
		self
	}
	pub fn flatscreen_pos_y(&mut self, flatscreen_pos_y: i32) -> &mut Self {
		self.settings.flatscreen_pos_y = flatscreen_pos_y;
		self
	}
	pub fn flatscreen_pos(&mut self, flatscreen_pos: (i32, i32)) -> &mut Self {
		self.settings.flatscreen_pos_x = flatscreen_pos.0;
		self.settings.flatscreen_pos_y = flatscreen_pos.1;
		self
	}
	pub fn flatscreen_width(&mut self, flatscreen_width: i32) -> &mut Self {
		self.settings.flatscreen_width = flatscreen_width;
		self
	}
	pub fn flatscreen_height(&mut self, flatscreen_height: i32) -> &mut Self {
		self.settings.flatscreen_height = flatscreen_height;
		self
	}
	pub fn flatscreen_size(&mut self, flatscreen_size: (i32, i32)) -> &mut Self {
		self.settings.flatscreen_width = flatscreen_size.0;
		self.settings.flatscreen_height = flatscreen_size.1;
		self
	}
	pub fn disable_flatscreen_mr_sim(&mut self, disable_flatscreen_mr_sim: bool) -> &mut Self {
		self.settings.disable_flatscreen_mr_sim = disable_flatscreen_mr_sim;
		self
	}
	pub fn disable_desktop_input_window(
		&mut self,
		disabled_desktop_input_window: bool,
	) -> &mut Self {
		self.settings.disable_desktop_input_window = disabled_desktop_input_window;
		self
	}
	pub fn disable_unfocused_sleep(&mut self, disable_unfocused_sleep: bool) -> &mut Self {
		self.settings.disable_unfocused_sleep = disable_unfocused_sleep;
		self
	}
	pub fn render_scaling(&mut self, render_scaling: f32) -> &mut Self {
		self.settings.render_scaling = render_scaling;
		self
	}
	pub fn origin (&mut self, origin_mode : OriginMode) -> &mut Self {
		self.settings.origin = origin_mode;
		self
	}
	fn build(&mut self) -> Settings {
		self.settings.clone()
	}
	pub fn init(&mut self) -> SkResult<SkSingle> {
		self.build().init()
	}
}

/// Information about a system’s capabilities and properties!
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
	/// The type of display this device has.
	pub display_type: Display,
	/// Width of the display surface, in pixels! For a stereo display, this will be the width of a single eye.
	pub display_width: i32,
	/// Height of the display surface, in pixels! For a stereo display, this will be the height of a single eye.
	pub display_height: i32,
	/// Does the device we’re currently on have the spatial graph bridge extension? The extension is provided through the function World.FromSpatialNode. This allows OpenXR to talk with certain windows APIs, such as the QR code API that provides Graph Node GUIDs for the pose.
	pub spatial_bridge_present: bool,
	/// Can the device work with externally provided spatial anchors, like UWP’s Windows.Perception.Spatial.SpatialAnchor
	pub perception_bridge_present: bool,
	/// Does the device we’re on have eye tracking support present? This is not an indicator that the user has given the application permission to access this information. See Input.Gaze for how to use this data.
	pub eye_tracking_present: bool,
	/// This tells if the app was successfully started as an overlay application. If this is true, then expect this application to be composited with other content below it!
	pub overlay_app: bool,
	/// Does this device support world occlusion of digital objects? If this is true, then World.OcclusionEnabled can be set to true, and World.OcclusionMaterial can be modified.
	pub world_occlusion_present: bool,
	/// Can this device get ray intersections from the environment? If this is true, then World.RaycastEnabled can be set to true, and World.Raycast can be used.
	pub world_raycast_present: bool,
}
impl From<system_info_t> for SystemInfo {
	fn from(value: system_info_t) -> Self {
		match value {
			system_info_t {
				display_type,
				display_width,
				display_height,
				spatial_bridge_present,
				perception_bridge_present,
				eye_tracking_present,
				overlay_app,
				world_occlusion_present,
				world_raycast_present,
			} => SystemInfo {
				display_type: unsafe { std::mem::transmute(display_type) },
				display_width,
				display_height,
				spatial_bridge_present: spatial_bridge_present != 0,
				perception_bridge_present: perception_bridge_present != 0,
				eye_tracking_present: eye_tracking_present != 0,
				overlay_app: overlay_app != 0,
				world_occlusion_present: world_occlusion_present != 0,
				world_raycast_present: world_raycast_present != 0,
			},
		}
	}
}

/// This tells about the app’s current focus state, whether it’s active and receiving input, or if it’s backgrounded or hidden. This can be important since apps may still run and render when unfocused, as the app may still be visible behind the app that does have focus.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum AppFocus {
	/// This StereoKit app is active, focused, and receiving input from the user. Application should behave as normal.
	Active = 0,
	/// This StereoKit app has been unfocused, something may be compositing on top of the app such as an OS dashboard. The app is still visible, but some other thing has focus and is receiving input. You may wish to pause, disable input tracking, or other such things.
	Background = 1,
	/// This app is not rendering currently.
	Hidden = 2,
}
impl From<app_focus_> for AppFocus {
	fn from(value: app_focus_) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}

/// StereoKit uses an asynchronous loading system to prevent assets from blocking execution! This means that asset loading systems will return an asset to you right away, even though it is still being processed in the background.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum AssetState {
	/// This asset encountered an issue when parsing the source data. Either the format is unrecognized by StereoKit, or the data may be corrupt. Check the logs for additional details.
	Unsupported = -3,
	/// The asset data was not found! This is most likely an issue with a bad file path, or file permissions. Check the logs for additional details.
	NotFound = -2,
	/// An unknown error occurred when trying to load the asset! Check the logs for additional details.
	Error = -1,
	/// This asset is in its default state. It has not been told to load anything, nor does it have any data!
	None = 0,
	/// This asset is currently queued for loading, but hasn’t received any data yet. Attempting to access metadata or asset data will result in blocking the app’s execution until that data is loaded!
	Loading = 1,
	/// This asset is still loading, but some of the higher level data is already available for inspection without blocking the app. Attempting to access the core asset data will result in blocking the app’s execution until that data is loaded!
	LoadedMeta = 2,
	/// This asset is completely loaded without issues, and is ready for use!
	Loaded = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum DeviceTracking {
	None = 0,
	Dof3 = 1,
	Dof6 = 2,
}
impl From<device_tracking_> for DeviceTracking {
	fn from(value: device_tracking_) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<device_tracking_> for DeviceTracking {
	fn into(self) -> device_tracking_ {
		unsafe { std::mem::transmute(self) }
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum DisplayType {
	None = 0,
	Stereo = 1,
	Flatscreen = 2,
}
impl From<display_type_> for DisplayType {
	fn from(value: display_type_) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}

pub type FovInfo = fov_info_t;

/// A position and a direction indicating a ray through space!
/// This is a great tool for intersection testing with geometrical
/// shapes.
///
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[repr(C)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::prelude::Reflect, bevy_reflect::prelude::FromReflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(Component))]
pub struct Ray {
	/// The position or origin point of the Ray.
	pub pos: Vec3,
	/// The direction the ray is facing, typically does not
	/// require being a unit vector, or normalized direction.
	pub dir: Vec3,
}
impl Ray {
	pub fn new(pos: impl Into<Vec3>, dir: impl Into<Vec3>) -> Self {
		Self {
			pos: pos.into(),
			dir: dir.into(),
		}
	}
}
impl From<ray_t> for Ray {
	fn from(value: ray_t) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<ray_t> for Ray {
	fn into(self) -> ray_t {
		unsafe { std::mem::transmute(self) }
	}
}

/// Bounds is an axis aligned bounding box type that can be used
/// for storing the sizes of objects, calculating containment,
/// intersections, and more!
///
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[repr(C)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::prelude::Reflect, bevy_reflect::prelude::FromReflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(Component))]
pub struct Bounds {
	/// The exact center of the Bounds!
	pub center: Vec3,
	/// The total size of the box, from one end to the other. This
	/// is the width, height, and depth of the Bounds.
	pub dimensions: Vec3,
}
impl From<bounds_t> for Bounds {
	fn from(value: bounds_t) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<bounds_t> for Bounds {
	fn into(self) -> bounds_t {
		unsafe { std::mem::transmute(self) }
	}
}

/// Planes are really useful for collisions, intersections, and
/// visibility testing!
///
/// This plane is stored using the ax + by + cz + d = 0 formula, where
/// the normal is a,b,c, and the d is, well, d.
///
#[derive(Debug, Copy, Clone, Default, Deserialize, Serialize)]
#[repr(C)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::prelude::Reflect, bevy_reflect::prelude::FromReflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(Component))]
pub struct Plane {
	/// The direction the plane is facing.
	pub normal: Vec3,
	/// The distance/travel along the plane's normal from
	/// the origin to the surface of the plane.
	pub d: f32,
}
impl From<plane_t> for Plane {
	fn from(value: plane_t) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<plane_t> for Plane {
	fn into(self) -> plane_t {
		unsafe { std::mem::transmute(self) }
	}
}

/// Represents a sphere in 3D space! Composed of a center point
/// and a radius, can be used for raycasting, collision, visibility, and
/// other things!
///
#[derive(Debug, Copy, Clone, Default, Deserialize, Serialize)]
#[repr(C)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::prelude::Reflect, bevy_reflect::prelude::FromReflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(Component))]
pub struct Sphere {
	/// Center of the sphere.
	pub center: Vec3,
	/// Distance from the center, to the surface of the sphere,
	/// in meters. Half the diameter.
	pub radius: f32,
}
impl From<sphere_t> for Sphere {
	fn from(value: sphere_t) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<sphere_t> for Sphere {
	fn into(self) -> sphere_t {
		unsafe { std::mem::transmute(self) }
	}
}

/// A Gradient is a sparse collection of color keys that are
/// used to represent a ramp of colors! This class is largely just
/// storing colors and allowing you to sample between them.
///
/// Since the Gradient is just interpolating values, you can use whatever
/// color space you want here, as long as it's linear and not gamma!
/// Gamma space RGB can't math properly at all. It can be RGB(linear),
/// HSV, LAB, just remember which one you have, and be sure to convert it
/// appropriately later. Data is stored as float colors, so this'll be a
/// high accuracy blend!
///


pub struct Gradient(pub NonNull<_gradient_t>);
impl Drop for Gradient {
	fn drop(&mut self) {
		gradient_release(self);
	}
}
impl AsRef<Gradient> for Gradient {
	fn as_ref(&self) -> &Gradient {
		&self
	}
}
unsafe impl Send for Gradient {}
unsafe impl Sync for Gradient {}


/// A Mesh is a single collection of triangular faces with extra surface
/// information to enhance rendering! StereoKit meshes are composed of a
/// list of vertices, and a list of indices to connect the vertices into
/// faces. Nothing more than that is stored here, so typically meshes are
/// combined with Materials, or added to Models in order to draw them.
///
/// Mesh vertices are composed of a position, a normal (direction of the
/// vert), a uv coordinate (for mapping a texture to the mesh's surface),
/// and a 32 bit color containing red, green, blue, and alpha
/// (transparency).
///
/// Mesh indices are stored as unsigned ints, so you can have a mesh with
/// a fudgeton of verts! 4 billion or so :)
pub struct Mesh(pub NonNull<_mesh_t>);
impl From<stereokit_sys::mesh_t> for Mesh {
	fn from(value: mesh_t) -> Self {
		Mesh(NonNull::new(value).unwrap())
	}
}
impl Drop for Mesh {
	fn drop(&mut self) {
		mesh_release(self)
	}
}
impl AsRef<Mesh> for Mesh {
	fn as_ref(&self) -> &Mesh {
		&self
	}
}
unsafe impl Send for Mesh {}
unsafe impl Sync for Mesh {}

/// This is the texture asset class! This encapsulates 2D images,
/// texture arrays, cubemaps, and rendertargets! It can load any image
/// format that stb_image can, (jpg, png, tga, bmp, psd, gif, hdr, pic)
/// plus more later on, and you can also create textures procedurally.
pub struct Tex(pub NonNull<_tex_t>);

impl AsRef<Tex> for Tex {
	fn as_ref(&self) -> &Tex {
		&self
	}
}

unsafe impl Send for Tex {}
unsafe impl Sync for Tex {}

/// This class represents a text font asset! On the back-end, this asset
/// is composed of a texture with font characters rendered to it, and a list of
/// data about where, and how large those characters are on the texture.
///
/// This asset is used anywhere that text shows up, like in the UI or Text classes!
pub struct Font(pub NonNull<_font_t>);
impl Drop for Font {
	fn drop(&mut self) {
		font_release(self)
	}
}
impl AsRef<Font> for Font {
	fn as_ref(&self) -> &Font {
		&self
	}
}

/// A shader is a piece of code that runs on the GPU, and
/// determines how model data gets transformed into pixels on screen!
/// It's more likely that you'll work more directly with Materials, which
/// shaders are a subset of.
///
/// With this particular class, you can mostly just look at it. It doesn't
/// do a whole lot. Maybe you can swap out the shader code or something
/// sometimes!
pub struct Shader(pub NonNull<_shader_t>);
impl Drop for Shader {
	fn drop(&mut self) {
		shader_release(self)
	}
}
impl AsRef<Shader> for Shader {
	fn as_ref(&self) -> &Shader {
		&self
	}
}

/// A Material describes the surface of anything drawn on the
/// graphics card! It is typically composed of a Shader, and shader
/// properties like colors, textures, transparency info, etc.
///
/// Items drawn with the same Material can be batched together into a
/// single, fast operation on the graphics card, so re-using materials
/// can be extremely beneficial for performance!
pub struct Material(pub NonNull<_material_t>);
impl Drop for Material {
	fn drop(&mut self) {
		material_release(self)
	}
}
impl AsRef<Material> for Material {
	fn as_ref(&self) -> &Material {
		&self
	}
}
unsafe impl Send for Material {}
unsafe impl Sync for Material {}

pub struct MaterialBuffer(pub NonNull<_material_buffer_t>);
impl Drop for MaterialBuffer {
	fn drop(&mut self) {
		material_buffer_release(self)
	}
}
impl AsRef<MaterialBuffer> for MaterialBuffer {
	fn as_ref(&self) -> &MaterialBuffer {
		&self
	}
}

/// A Model is a collection of meshes, materials, and transforms
/// that make up a visual element! This is a great way to group together
/// complex objects that have multiple parts in them, and in fact, most
/// model formats are composed this way already!
///
/// This class contains a number of methods for creation. If you pass in
/// a .obj, .stl, , .ply (ASCII), .gltf, or .glb, StereoKit will load
/// that model from file, and assemble materials and transforms from the
/// file information. But you can also assemble a model from procedurally
/// generated meshes!
///
/// Because models include an offset transform for each mesh element,
/// this does have the overhead of an extra matrix multiplication in
/// order to execute a render command. So if you need speed, and only
/// have a single mesh with a precalculated transform matrix, it can be
/// faster to render a Mesh instead of a Model!
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::prelude::Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(Component))]
pub struct Model(
	#[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
	pub _Model
);
impl AsRef<Model> for Model {
	fn as_ref(&self) -> &Model {
		&self
	}
}
impl Drop for Model {
	fn drop(&mut self) {
		model_release(self);
	}
}
impl Model {
	pub fn from(arg: NonNull<_model_t>) -> Self {
		Self(_Model(arg))
	}
}
unsafe impl Send for _Model {}
unsafe impl Sync for _Model {}

pub struct _Model(pub NonNull<_model_t>);

impl _Model {
	pub fn as_ptr(&self) -> *mut _model_t {
		self.0.as_ptr()
	}
}

#[cfg(feature = "bevy_reflect")]
impl Default for Model {
	fn default() -> Self {
		let sk = unsafe { Sk::create_unsafe() };
		sk.model_create_mesh(Mesh::CUBE, Material::DEFAULT)
	}
}

#[cfg(feature = "bevy_reflect")]
impl FromReflect for Model {
	fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
		let sk = unsafe { Sk::create_unsafe()};
		Some(sk.model_copy(reflect.downcast_ref::<Model>()?))
	}
}

/// A Sprite is an image that's set up for direct 2D rendering,
/// without using a mesh or model! This is technically a wrapper over a
/// texture, but it also includes atlasing functionality, which can be
/// pretty important to performance! This is used a lot in UI, for image
/// rendering.
///
/// Atlasing is not currently implemented, it'll swap to Single for now.
/// But here's how it works!
///
/// StereoKit will batch your sprites into an atlas if you ask it to!
/// This puts all the images on a single texture to significantly reduce
/// draw calls when many images are present. Any time you add a sprite to
/// an atlas, it'll be marked as dirty and rebuilt at the end of the
/// frame. So it can be a good idea to add all your images to the atlas
/// on initialize rather than during execution!
///
/// Since rendering is atlas based, you also have only one material per
/// atlas. So this is why you might wish to put a sprite in one atlas or
/// another, so you can apply different
pub struct Sprite(pub NonNull<_sprite_t>);
impl Drop for Sprite {
	fn drop(&mut self) {
		sprite_release(self)
	}
}
impl AsRef<Sprite> for Sprite {
	fn as_ref(&self) -> &Sprite {
		&self
	}
}

/// This class represents a sound effect! Excellent for blips
/// and bloops and little clips that you might play around your scene.
/// Right now, this supports .wav, .mp3, and procedurally generated
/// noises!
///
/// On HoloLens 2, sounds are automatically processed on the HPU, freeing
/// up the CPU for more of your app's code. To simulate this same effect
/// on your development PC, you need to enable spatial sound on your
/// audio endpoint. To do this, right click the speaker icon in your
/// system tray, navigate to "Spatial sound", and choose "Windows Sonic
/// for Headphones." For more information, visit
/// <https://docs.microsoft.com/en-us/windows/win32/coreaudio/spatial-sound>
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
pub struct Sound(pub NonNull<_sound_t>);
impl Drop for Sound {
	fn drop(&mut self) {
		sound_release(self)
	}
}
impl AsRef<Sound> for Sound {
	fn as_ref(&self) -> &Sound {
		&self
	}
}
unsafe impl Send for Sound {}
unsafe impl Sync for Sound {}

/// A Solid is an object that gets simulated with physics! Once
/// you create a solid, it will continuously be acted upon by forces like
/// gravity and other objects. Solid does -not- draw anything on its own,
/// but you can ask a Solid for its current pose, and draw an object at
/// that pose!
///
/// Once you create a Solid, you need to define its shape using geometric
/// primitives, this is the AddSphere, AddCube, AddCapsule functions. You
/// can add more than one to a single Solid to get a more complex shape!
///
/// If you want to directly move a Solid, note the difference between the
/// Move function and the Teleport function. Move will change the
/// velocity for a single frame to travel through space to get to its
/// destination, while teleport will simply appear at its destination
/// without touching anything between.
pub struct Solid(pub NonNull<_solid_t>);

pub struct Asset(pub NonNull<std::os::raw::c_void>);
impl AsRef<Asset> for Asset {
	fn as_ref(&self) -> &Asset {
		&self
	}
}

impl From<Model> for Asset {
	fn from(value: Model) -> Self {
		unsafe { std::mem::transmute(value)}
	}
}

impl AsRef<Asset> for Model {
	fn as_ref(&self) -> &Asset {
		unsafe {std::mem::transmute(self)}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct TextStyle(pub u32);
/// A enum for describing alignment or positioning
#[derive(Debug, Copy, Clone, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum TextAlign {
	XLeft = 1,
	YTop = 2,
	XCenter = 4,
	YCenter = 8,
	XRight = 16,
	YBottom = 32,
	Center = 12,
	CenterLeft = 9,
	CenterRight = 24,
	Left = 3,
	Right = 18,
	BottomCenter = 36,
	BottomLeft = 33,
	BottomRight = 48,
}

/// This enum describes how text layout behaves within the space it is given.
#[derive(Debug, Copy, Clone, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum TextFit {
	/// The text will wrap around to the next line down when it reaches the end of the space on the X axis.
	Wrap = 1,
	/// When the text reaches the end, it is simply truncated and no longer visible.
	Clip = 2,
	/// If the text is too large to fit in the space provided, it will be scaled down to fit inside. This will not scale up.
	Squeeze = 4,
	/// If the text is larger, or smaller than the space provided, it will scale down or up to fill the space.
	Exact = 8,
	/// The text will ignore the containing space, and just keep on going.
	Overflow = 16,
}

impl Into<text_fit_> for TextFit {
	fn into(self) -> text_fit_ {
		unsafe { std::mem::transmute(self) }
	}
}

impl Into<text_align_> for TextAlign {
	fn into(self) -> text_align_ {
		unsafe { std::mem::transmute(self) }
	}
}

/// A color/position pair for Gradient values!
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct GradientKey {
	pub color: Color128,
	pub position: f32,
}
impl From<gradient_key_t> for GradientKey {
	fn from(value: gradient_key_t) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<gradient_key_t> for GradientKey {
	fn into(self) -> gradient_key_t {
		unsafe { std::mem::transmute(self) }
	}
}

/// A light source used for creating SphericalHarmonics data.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ShLight {
	/// Direction to the light source.
	pub dir_to: Vec3,
	/// Color of the light in linear space! Values here can exceed 1.
	pub color: Color128,
}
impl From<sh_light_t> for ShLight {
	fn from(value: sh_light_t) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<sh_light_t> for ShLight {
	fn into(self) -> sh_light_t {
		unsafe { std::mem::transmute(self) }
	}
}
/// Spherical Harmonics are kinda like Fourier, but on a sphere. That doesn’t mean terribly much to me, and could be wrong, but check out here for more details about how Spherical Harmonics work in this context!
///
/// However, the more prctical thing is, SH can be a function that describes a value over the surface of a sphere! This is particularly useful for lighting, since you can basically store the lighting information for a space in this value! This is often used for lightmap data, or a light probe grid, but StereoKit just uses a single SH for the entire scene. It’s a gross oversimplification, but looks quite good, and is really fast! That’s extremely great when you’re trying to hit 60fps, or even 144fps.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct SphericalHarmonics {
	pub coefficients: [Vec3; 9usize],
}
impl From<spherical_harmonics_t> for SphericalHarmonics {
	fn from(value: spherical_harmonics_t) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<spherical_harmonics_t> for SphericalHarmonics {
	fn into(self) -> spherical_harmonics_t {
		unsafe { std::mem::transmute(self) }
	}
}

/// This represents a single vertex in a Mesh, all StereoKit Meshes currently use this exact layout!
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Vert {
	/// Position of the vertex, in model space coordinates.
	pub pos: Vec3,
	/// The normal of this vertex, or the direction the vertex is facing. Preferably normalized.
	pub norm: Vec3,
	/// The texture coordinates at this vertex.
	pub uv: Vec2,
	/// The color of the vertex. If you aren’t using it, set it to white.
	pub col: Color32,
}
impl From<vert_t> for Vert {
	fn from(value: vert_t) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<vert_t> for Vert {
	fn into(self) -> vert_t {
		unsafe { std::mem::transmute(self) }
	}
}

///Culling is discarding an object from the render pipeline! This enum describes how mesh faces get discarded on the graphics card. With culling set to none, you can double the number of pixels the GPU ends up drawing, which can have a big impact on performance. None can be appropriate in cases where the mesh is designed to be ‘double sided’. Front can also be helpful when you want to flip a mesh ‘inside-out’!
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum CullMode {
	/// Discard if the back of the triangle face is pointing towards the camera. This is the default behavior.
	Back = 0,
	/// Discard if the front of the triangle face is pointing towards the camera. This is opposite the default behavior.
	Front = 1,
	/// No culling at all! Draw the triangle regardless of which way it’s pointing.
	None = 2,
}

bitflags::bitflags! {
	/// Textures come in various types and flavors! These are bit-flags
	/// that tell StereoKit what type of texture we want; and how the application
	/// might use it!
	#[derive(Serialize, Deserialize)]
	pub struct TextureType: u32 {
		/// A standard color image; without any generated mip-maps.
		const IMAGE_NO_MIPS = 1 << 0;
		/// A size sided texture that's used for things like skyboxes;
		/// environment maps; and reflection probes. It behaves like a texture
		/// array with 6 textures.
		const CUBEMAP = 1 << 1;
		/// This texture can be rendered to! This is great for textures
		/// that might be passed in as a target to Renderer.Blit; or other
		/// such situations.
		const RENDER_TARGET = 1 << 2;
		/// This texture contains depth data; not color data!
		const DEPTH = 1 << 3;
		/// This texture will generate mip-maps any time the contents
		/// change. Mip-maps are a list of textures that are each half the
		/// size of the one before them! This is used to prevent textures from
		/// 'sparkling' or aliasing in the distance.
		const MIPS = 1 << 4;
		/// This texture's data will be updated frequently from the
		/// CPU (not renders)! This ensures the graphics card stores it
		/// someplace where writes are easy to do quickly.
		const DYNAMIC = 1 << 5;
		/// A standard color image that also generates mip-maps
		/// automatically.
		const IMAGE = Self::IMAGE_NO_MIPS.bits | Self::MIPS.bits;
	}
}

/// What type of color information will the texture contain? A
/// good default here is Rgba32.
#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureFormat {
	/// A default zero value for TexFormat! Uninitialized formats
	/// will get this value and **** **** up so you know to assign it
	/// properly :)
	None = 0,
	/// Red/Green/Blue/Transparency data channels, at 8 bits
	/// per-channel in sRGB color space. This is what you'll want most of
	/// the time you're dealing with color images! Matches well with the
	/// Color32 struct! If you're storing normals, rough/metal, or
	/// anything else, use Rgba32Linear.
	RGBA32 = 1,
	/// Red/Green/Blue/Transparency data channels, at 8 bits
	/// per-channel in linear color space. This is what you'll want most
	/// of the time you're dealing with color data! Matches well with the
	/// Color32 struct.
	RGBA32Linear = 2,
	BGRA32 = 3,
	BGRA32Linear = 4,
	RG11B10 = 5,
	RGB10A2 = 6,
	/// Red/Green/Blue/Transparency data channels, at 16 bits
	/// per-channel! This is not common, but you might encounter it with
	/// raw photos, or HDR images.
	RGBA64 = 7, // TODO: remove during major version update
	RGBA64S = 8,
	RGBA64F = 9,
	/// Red/Green/Blue/Transparency data channels at 32 bits
	/// per-channel! Basically 4 floats per color, which is bonkers
	/// expensive. Don't use this unless you know -exactly- what you're
	/// doing.
	RGBA128 = 10,
	/// A single channel of data, with 8 bits per-pixel! This
	/// can be great when you're only using one channel, and want to
	/// reduce memory usage. Values in the shader are always 0.0-1.0.
	R8 = 11,
	/// A single channel of data, with 16 bits per-pixel! This
	/// is a good format for height maps, since it stores a fair bit of
	/// information in it. Values in the shader are always 0.0-1.0.
	R16 = 12,
	/// A single channel of data, with 32 bits per-pixel! This
	/// basically treats each pixel as a generic float, so you can do all
	/// sorts of strange and interesting things with this.
	R32 = 13,
	/// A depth data format, 24 bits for depth data, and 8 bits
	/// to store stencil information! Stencil data can be used for things
	/// like clipping effects, deferred rendering, or shadow effects.
	DepthStencil = 14,
	/// 32 bits of data per depth value! This is pretty detailed,
	/// and is excellent for experiences that have a very far view
	/// distance.
	Depth32 = 15,
	/// 16 bits of depth is not a lot, but it can be enough if
	/// your far clipping plane is pretty close. If you're seeing lots of
	/// flickering where two objects overlap, you either need to bring
	/// your far clip in, or switch to 32/24 bit depth.
	Depth16 = 16,
}

/// How does the shader grab pixels from the texture? Or more
/// specifically, how does the shader grab colors between the provided
/// pixels? If you'd like an in-depth explanation of these topics, check
/// out [this exploration of texture filtering](https://medium.com/@bgolus/sharper-mipmapping-using-shader-based-supersampling-ed7aadb47bec)
/// by graphics wizard Ben Golus.
#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureSample {
	/// Use a linear blend between adjacent pixels, this creates
	/// a smooth, blurry look when texture resolution is too low.
	Linear = 0,
	/// Choose the nearest pixel's color! This makes your texture
	/// look like pixel art if you're too close.
	Point = 1,
	/// This helps reduce texture blurriness when a surface is
	/// viewed at an extreme angle!
	Anisotropic = 2,
}

/// What happens when the shader asks for a texture coordinate
/// that's outside the texture?? Believe it or not, this happens plenty
/// often!
#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureAddress {
	/// Wrap the UV coordinate around to the other side of the
	/// texture! This is basically like a looping texture, and is an
	/// excellent default. If you can see weird bits of color at the edges
	/// of your texture, this may be due to Wrap blending the color with
	/// the other side of the texture, Clamp may be better in such cases.
	Wrap = 0,
	/// Clamp the UV coordinates to the edge of the texture!
	/// This'll create color streaks that continue to forever. This is
	/// actually really great for non-looping textures that you know will
	/// always be accessed on the 0-1 range.
	Clamp = 1,
	/// Like Wrap, but it reflects the image each time! Who needs
	/// this? I'm not sure!! But the graphics card can do it, so now you
	/// can too!
	Mirror = 2,
}

/// Also known as ‘alpha’ for those in the know. But there’s actually more than one type of transparency in rendering! The horrors. We’re keepin’ it fairly simple for now, so you get three options!
#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum Transparency {
	/// Not actually transparent! This is opaque! Solid! It’s the default option, and it’s the fastest option! Opaque objects write to the z-buffer, the occlude pixels behind them, and they can be used as input to important Mixed Reality features like Late Stage Reprojection that’ll make your view more stable!
	None = 1,
	/// This will blend with the pixels behind it. This is transparent! You may not want to write to the z-buffer, and it’s slower than opaque materials.
	Blend = 2,
	/// This will straight up add the pixel color to the color buffer! This usually looks -really- glowy, so it makes for good particles or lighting effects.
	Add = 3,
}

/// Depth test describes how this material looks at and responds to depth information in the zbuffer! The default is Less, which means if the material pixel’s depth is Less than the existing depth data, (basically, is this in front of some other object) it will draw that pixel. Similarly, Greater would only draw the material if it’s ‘behind’ the depth buffer. Always would just draw all the time, and not read from the depth buffer at all.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum DepthTest {
	/// Default behavior, pixels behind the depth buffer will be discarded, and pixels in front of it will be drawn.
	Less = 0,
	/// Pixels behind the depth buffer will be discarded, and pixels in front of, or at the depth buffer’s value it will be drawn. This could be great for things that might be sitting exactly on a floor or wall.
	LessOrEq = 1,
	/// Pixels in front of the zbuffer will be discarded! This is opposite of how things normally work. Great for drawing indicators that something is occluded by a wall or other geometry.
	Greater = 2,
	/// Pixels in front of (or exactly at) the zbuffer will be discarded! This is opposite of how things normally work. Great for drawing indicators that something is occluded by a wall or other geometry.
	GreaterOrEq = 3,
	/// Only draw pixels if they’re at exactly the same depth as the zbuffer!
	Equal = 4,
	/// Draw any pixel that’s not exactly at the value in the zbuffer.
	NotEqual = 5,
	/// Don’t look at the zbuffer at all, just draw everything, always, all the time! At this point, the order at which the mesh gets drawn will be super important, so don’t forget about Material.QueueOffset!
	Always = 6,
	/// Never draw a pixel, regardless of what’s in the zbuffer. I can think of better ways to do this, but uhh, this is here for completeness! Maybe you can find a use for it.
	Never = 7,
}

/// TODO: v0.4 This may need significant revision? What type of data does this material parameter need? This is used to tell the shader how large the data is, and where to attach it to on the shader.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum MaterialParameter {
	/// This data type is not currently recognized. Please report your case on GitHub Issues!
	Unknown = 0,
	Float = 1,
	Color128 = 2,
	Vec2 = 3,
	Vec3 = 4,
	Vec4 = 5,
	/// A 4x4 matrix of floats.
	Matrix = 6,
	///	Texture information!
	Texture = 7,
	Int = 8,
	Int2 = 9,
	Int3 = 10,
	Int4 = 11,
	UInt = 12,
	UInt2 = 13,
	UInt3 = 14,
	UInt4 = 15,
}

/// Describes how an animation is played back, and what to do when the animation hits the end.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum AnimMode {
	/// If the animation reaches the end, it will always loop back around to the start again.
	Loop = 0,
	/// When the animation reaches the end, it will freeze in-place.
	Once = 1,
	/// The animation will not progress on its own, and instead must be driven by providing information to the model’s AnimTime or AnimCompletion properties.
	Manual = 2,
}

/// The way the Sprite is stored on the backend! Does it get batched and atlased for draw efficiency, or is it a single image?
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum SpriteType {
	/// The sprite will be batched onto an atlas texture so all sprites can be drawn in a single pass. This is excellent for performance! The only thing to watch out for here, adding a sprite to an atlas will rebuild the atlas texture! This can be a bit expensive, so it’s recommended to add all sprites to an atlas at start, rather than during runtime. Also, if an image is too large, it may take up too much space on the atlas, and may be better as a Single sprite type.
	Atlased = 0,
	/// This sprite is on its own texture. This is best for large images, items that get loaded and unloaded during runtime, or for sprites that may have edge artifacts or severe ‘bleed’ from adjacent atlased images.
	Single = 1,
}

/// Used to represent lines for the line drawing functions! This is just a snapshot of information about each individual point on a line.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct LinePoint {
	/// Location of the line point
	pub pt: Vec3,
	/// Total thickness of the line, in meters.
	pub thickness: f32,
	/// The vertex color for the line at this position.
	pub color: Color32,
}
impl From<line_point_t> for LinePoint {
	fn from(value: line_point_t) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
impl Into<line_point_t> for LinePoint {
	fn into(self) -> line_point_t {
		unsafe { std::mem::transmute(self) }
	}
}

/// Pose represents a location and orientation in space, excluding scale!
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[repr(C)]
pub struct Pose {
	pub position: Vec3,
	pub orientation: Quat,
}
impl AsMut<Pose> for Pose {
	fn as_mut(&mut self) -> &mut Pose {
		self
	}
}
impl Pose {
	pub const IDENTITY: Pose = Pose {
		position: Vec3::new(0.0, 0.0, 0.0),
		orientation: Quat::IDENTITY,
	};
	pub fn new(position: impl Into<Vec3>, orientation: impl Into<Quat>) -> Self {
		Self {
			position: position.into(),
			orientation: orientation.into(),
		}
	}
	/// Calculates the forward direction from this pose. This is done by multiplying the orientation with Vec3::new(0, 0, -1). Remember that Forward points down the -Z axis!
	pub fn forward(&self) -> Vec3 {
		self.orientation.mul_vec3(Vec3::new(0.0, 0.0, -1.0))
	}

	/// This creates a ray starting at the Pose’s position, and pointing in the ‘Forward’ direction. The Ray direction is a unit vector/normalized.
	pub fn ray(&self) -> Ray {
		Ray {
			pos: self.position,
			dir: Vec3::new(0.0, 0.0, -1.0),
		}
	}
}
impl Default for Pose {
	fn default() -> Self {
		Pose::IDENTITY
	}
}
impl From<pose_t> for Pose {
	fn from(value: pose_t) -> Self {
		Pose {
			position: value.position.into(),
			orientation: Quat::from_xyzw(
				value.orientation.x,
				value.orientation.y,
				value.orientation.z,
				value.orientation.w,
			),
		}
	}
}
impl Into<pose_t> for Pose {
	fn into(self) -> pose_t {
		pose_t {
			position: self.position.into(),
			orientation: quat {
				x: self.orientation.x,
				y: self.orientation.y,
				z: self.orientation.z,
				w: self.orientation.w,
			},
		}
	}
}

/// When rendering to a rendertarget, this tells if and what of the rendertarget gets cleared before rendering. For example, if you are assembling a sheet of images, you may want to clear everything on the first image draw, but not clear on subsequent draws.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum RenderClear {
	/// Don’t clear anything, leave it as it is.
	None = 0,
	/// Clear the rendertarget’s color data.
	Color = 1,
	/// Clear the rendertarget’s depth data, if present.
	Depth = 2,
	/// Clear both color and depth data.
	All = 3,
}
impl Into<render_clear_> for RenderClear {
	fn into(self) -> render_clear_ {
		self as render_clear_
	}
}

/// The projection mode used by StereoKit for the main camera! You can use this with Renderer.Projection. These options are only available in flatscreen mode, as MR headsets provide very specific projection matrices.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum Projection {
	/// This is the default projection mode, and the one you’re most likely to be familiar with! This is where parallel lines will converge as they go into the distance.
	Perspective = 0,
	/// Orthographic projection mode is often used for tools, 2D rendering, thumbnails of 3D objects, or other similar cases. In this mode, parallel lines remain parallel regardless of how far they travel.
	Orthographic = 1,
}
impl Into<projection_> for Projection {
	fn into(self) -> projection_ {
		self as projection_
	}
}

pub type Rect = rect_t;
/// This represents a play instance of a Sound! You can get one when you call Sound.Play(). This allows you to do things like cancel a piece of audio early, or change the volume and position of it as it’s playing.
pub type SoundInstance = sound_inst_t;

bitflags::bitflags! {
/// What type of device is the source of the pointer? This is a bit-flag that can contain some input source family information.
	#[derive(Deserialize, Serialize)]
	pub struct InputSource: u32 {
		/// Matches with all input sources!
		const ANY = 2147483647;
		/// Matches with any hand input source.
		const HAND = 1;
		/// Matches with left hand input sources.
		const HAND_LEFT = 2;
		/// Matches with right hand input sources.
		const HAND_RIGHT = 4;
		/// Matches with Gaze category input sources.
		const GAZE = 16;
		/// Matches with the head gaze input source.
		const GAZE_HEAD = 32;
		/// Matches with the eye gaze input source.
		const GAZE_EYES = 64;
		/// Matches with mouse cursor simulated gaze as an input source.
		const GAZE_CURSOR = 128;
		/// Matches with any input source that has an activation button!
		const CAN_PRESS = 256;
	}
}
/// An enum for indicating which hand to use!
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum Handed {
	/// Left hand.
	Left = 0,
	/// Right hand.
	Right = 1,
	/// The number of hands one generally has, this is much nicer than doing a for loop with ‘2’ as the condition! It’s much clearer when you can loop Hand.Max times instead.
	Max = 2,
}
bitflags::bitflags! {
	/// A bit-flag for the current state of a button input.
	/// This is *BROKEN* you cannot rely on Inactive to be false, I am waiting for nick to fix it lmao
	#[derive(Deserialize, Serialize)]
	pub struct ButtonState: u32 {
		/// Is the button currently up, unpressed?
		const INACTIVE = 0;
		/// Is the button currently down, pressed?
		const ACTIVE = 1;
		///	Has the button just been released? Only true for a single frame.
		const JUST_INACTIVE = 2;
		///	Has the button just been pressed? Only true for a single frame.
		const JUST_ACTIVE = 4;
		/// Has the button just changed state this frame?
		const CHANGED = 6;
		/// Matches with all states!
		const ANY = 2147483647;
	}
}
/// This is the tracking state of a sensory input in the world, like a controller’s position sensor, or a QR code identified by a tracking system.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum TrackState {
	/// The system has no current knowledge about the state of this input. It may be out of visibility, or possibly just disconnected.
	Lost = 0,
	/// The system doesn’t know for sure where this is, but it has an educated guess that may be inferred from previous data at a lower quality. For example, a controller may still have accelerometer data after going out of view, which can still be accurate for a short time after losing optical tracking.
	Inferred = 1,
	/// The system actively knows where this input is. Within the constraints of the relevant hardware’s capabilities, this is as accurate as it gets!
	Known = 2,
}

/// Pointer is an abstraction of a number of different input sources, and a way to surface input events!
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct Pointer {
	/// What input source did this pointer come from? This is a bit-flag that contains input family and capability information.
	pub source: InputSource,
	/// Is the pointer source being tracked right now?
	pub tracked: ButtonState,
	/// What is the state of the input source’s ‘button’, if it has one?
	pub state: ButtonState,
	/// A ray in the direction of the pointer.
	pub ray: Ray,
	/// Orientation of the pointer! Since a Ray has no concept of ‘up’, this can be used to retrieve more orientation information.
	pub orientation: Quat,
}
impl Into<pointer_t> for Pointer {
	fn into(self) -> pointer_t {
		match self {
			Pointer {
				source,
				tracked,
				state,
				ray,
				orientation,
			} => pointer_t {
				source: source.bits as IntegerType,
				tracked: tracked.bits as IntegerType,
				state: state.bits as IntegerType,
				ray: ray.into(),
				orientation: quat {
					x: orientation.x,
					y: orientation.y,
					z: orientation.z,
					w: orientation.w,
				},
			},
		}
	}
}
impl From<pointer_t> for Pointer {
	fn from(value: pointer_t) -> Self {
		match value {
			pointer_t {
				source,
				tracked,
				state,
				ray,
				orientation,
			} => Self {
				source: unsafe { InputSource::from_bits_unchecked(source as u32) },
				tracked: unsafe { ButtonState::from_bits_unchecked(tracked as u32) },
				state: unsafe { ButtonState::from_bits_unchecked(state as u32) },
				ray: ray.into(),
				orientation: Quat::from_xyzw(
					orientation.x,
					orientation.y,
					orientation.z,
					orientation.w,
				),
			},
		}
	}
}

/// Contains information to represents a joint on the hand.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct HandJoint {
	/// The center of the joint’s world space location.
	pub position: Vec3,
	/// The joint’s world space orientation, where Forward points to the next joint down the finger, and Up will point towards the back of the hand. On the left hand, Right will point towards the thumb, and on the right hand, Right will point away from the thumb.
	pub orientation: Quat,
	/// The distance, in meters, to the surface of the hand from this joint.
	pub radius: f32,
}
impl Into<hand_joint_t> for HandJoint {
	fn into(self) -> hand_joint_t {
		match self {
			HandJoint {
				position,
				orientation,
				radius,
			} => hand_joint_t {
				position: position.into(),
				orientation: quat {
					x: orientation.x,
					y: orientation.y,
					z: orientation.z,
					w: orientation.w,
				},
				radius,
			},
		}
	}
}
impl From<hand_joint_t> for HandJoint {
	fn from(value: hand_joint_t) -> Self {
		match value {
			hand_joint_t {
				position,
				orientation,
				radius,
			} => Self {
				position: position.into(),
				orientation: Quat::from_xyzw(
					orientation.x,
					orientation.y,
					orientation.z,
					orientation.w,
				),
				radius,
			},
		}
	}
}
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Hand {
	/// This is a 2D array with 25 HandJoints. You can get the right joint by finger*5 + joint
	pub fingers: [[HandJoint; 5]; 5],
	/// Pose of the wrist. TODO: Not populated right now.
	pub wrist: Pose,
	/// The position and orientation of the palm! Position is specifically defined as the middle of the middle finger’s root (metacarpal) bone. For orientation, Forward is the direction the flat of the palm is facing, “Iron Man” style. X+ is to the outside of the right hand, and to the inside of the left hand.
	pub palm: Pose,
	/// This is an approximation of where the center of a ‘pinch’ gesture occurs, and is used internally by StereoKit for some tasks, such as UI. For simulated hands, this position will give you the most stable pinch location possible. For real hands, it’ll be pretty close to the stablest point you’ll get. This is especially important for when the user begins and ends their pinch action, as you’ll often see a lot of extra movement in the fingers then.
	pub pinch_pt: Vec3,
	/// Is this a right hand, or a left hand?
	pub handedness: Handed,
	/// Is the hand being tracked by the sensors right now?
	pub tracked_state: ButtonState,
	/// Is the hand making a pinch gesture right now? Finger and thumb together.
	pub pinch_state: ButtonState,
	/// Is the hand making a grip gesture right now? Fingers next to the palm.
	pub grip_state: ButtonState,
	/// This is the size of the hand, calculated by measuring the length of the middle finger! This is calculated by adding the distances between each joint, then adding the joint radius of the root and tip. This value is recalculated at relatively frequent intervals, and can vary by as much as a centimeter.
	pub size: f32,
	/// What percentage of activation is the pinch gesture right now? Where 0 is a hand in an outstretched resting position, and 1 is fingers touching, within a device error tolerant threshold.
	pub pinch_activation: f32,
	/// What percentage of activation is the grip gesture right now? Where 0 is a hand in an outstretched resting position, and 1 is ring finger touching the base of the palm, within a device error tolerant threshold.
	pub grip_activation: f32,
}
impl Into<hand_t> for Hand {
	fn into(self) -> hand_t {
		match self {
			Hand {
				fingers,
				wrist,
				palm,
				pinch_pt,
				handedness,
				tracked_state,
				pinch_state,
				grip_state,
				size,
				pinch_activation,
				grip_activation,
			} => hand_t {
				fingers: fingers.map(|f| f.map(|f| f.into())),
				wrist: wrist.into(),
				palm: palm.into(),
				pinch_pt: pinch_pt.into(),
				handedness: handedness as handed_,
				tracked_state: tracked_state.bits as IntegerType,
				pinch_state: pinch_state.bits as IntegerType,
				grip_state: grip_state.bits as IntegerType,
				size,
				pinch_activation,
				grip_activation,
			},
		}
	}
}
impl From<hand_t> for Hand {
	fn from(value: hand_t) -> Self {
		match value {
			hand_t {
				fingers,
				wrist,
				palm,
				pinch_pt,
				handedness,
				tracked_state,
				pinch_state,
				grip_state,
				size,
				pinch_activation,
				grip_activation,
			} => Self {
				fingers: fingers.map(|f| f.map(|f| f.into())),
				wrist: wrist.into(),
				palm: palm.into(),
				pinch_pt: pinch_pt.into(),
				handedness: unsafe { std::mem::transmute(handedness) },
				tracked_state: unsafe { ButtonState::from_bits_unchecked(tracked_state as u32) },
				pinch_state: unsafe { ButtonState::from_bits_unchecked(pinch_state as u32) },
				grip_state: unsafe { ButtonState::from_bits_unchecked(grip_state as u32) },
				size,
				pinch_activation,
				grip_activation,
			},
		}
	}
}
/// This represents a physical controller input device! Tracking information, buttons, analog sticks and triggers! There’s also a Menu button that’s tracked separately at Input.ContollerMenu.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Controller {
	/// The grip pose of the controller. This approximately represents the center of the hand’s position. Check trackedPos and trackedRot for the current state of the pose data.
	pub pose: Pose,
	pub palm: Pose,
	/// The aim pose of a controller is where the controller ‘points’ from and to. This is great for pointer rays and far interactions.
	pub aim: Pose,
	/// This tells the current tracking state of this controller overall. If either position or rotation are trackable, then this will report tracked. Typically, positional tracking will be lost first, when the controller goes out of view, and rotational tracking will often remain as long as the controller is still connected. This is a good way to check if the controller is connected to the system at all.
	pub tracked: ButtonState,
	/// This tells the current tracking state of the controller’s position information. This is often the first part of tracking data to go, so it can often be good to account for this on occasions.
	pub tracked_pos: TrackState,
	/// This tells the current tracking state of the controller’s rotational information.
	pub tracked_rot: TrackState,
	/// This represents the click state of the controller’s analog stick or directional controller.
	pub stick_click: ButtonState,
	/// The current state of the controller’s X1 button. Depending on the specific hardware, this is the first general purpose button on the controller. For example, on an Oculus Quest Touch controller this would represent ‘X’ on the left controller, and ‘A’ on the right controller.
	pub x1: ButtonState,
	///The current state of the controller’s X2 button. Depending on the specific hardware, this is the second general purpose button on the controller. For example, on an Oculus Quest Touch controller this would represent ‘Y’ on the left controller, and ‘B’ on the right controller.
	pub x2: ButtonState,
	/// The trigger button at the user’s index finger. These buttons typically have a wide range of activation, so this is provided as a value from 0.0 -> 1.0, where 0 is no interaction, and 1 is full interaction. If a controller has binary activation, this will jump straight from 0 to 1.
	pub trigger: f32,
	/// The grip button typically sits under the user’s middle finger. These buttons occasionally have a wide range of activation, so this is provided as a value from 0.0 -> 1.0, where 0 is no interaction, and 1 is full interaction. If a controller has binary activation, this will jump straight from 0 to 1.
	pub grip: f32,
	/// This is the current 2-axis position of the analog stick or equivalent directional controller. This generally ranges from -1 to +1 on each axis. This is a raw input, so dead-zones and similar issues are not accounted for here, unless modified by the OpenXR platform itself.
	pub stick: Vec2,
}
impl Into<controller_t> for Controller {
	fn into(self) -> controller_t {
		match self {
			Controller {
				pose,
				palm,
				aim,
				tracked,
				tracked_pos,
				tracked_rot,
				stick_click,
				x1,
				x2,
				trigger,
				grip,
				stick,
			} => controller_t {
				pose: pose.into(),
				palm: palm.into(),
				aim: aim.into(),
				tracked: tracked.bits as IntegerType,
				tracked_pos: tracked_pos as track_state_,
				tracked_rot: tracked_rot as track_state_,
				stick_click: stick_click.bits as IntegerType,
				x1: x1.bits as IntegerType,
				x2: x2.bits as IntegerType,
				trigger,
				grip,
				stick: stick.into(),
			},
		}
	}
}
impl From<controller_t> for Controller {
	fn from(value: controller_t) -> Self {
		match value {
			controller_t {
				pose,
				palm,
				aim,
				tracked,
				tracked_pos,
				tracked_rot,
				stick_click,
				x1,
				x2,
				trigger,
				grip,
				stick,
			} => Self {
				pose: pose.into(),
				palm: palm.into(),
				aim: aim.into(),
				tracked: unsafe { ButtonState::from_bits_unchecked(tracked as u32) },
				tracked_pos: unsafe { std::mem::transmute(tracked_pos) },
				tracked_rot: unsafe { std::mem::transmute(tracked_rot) },
				stick_click: unsafe { ButtonState::from_bits_unchecked(stick_click as u32) },
				x1: unsafe { ButtonState::from_bits_unchecked(x1 as u32) },
				x2: unsafe { ButtonState::from_bits_unchecked(x2 as u32) },
				trigger,
				grip,
				stick: stick.into(),
			},
		}
	}
}
/// This stores information about the mouse! What’s its state, where’s it pointed, do we even have one?
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct Mouse {
	/// Is the mouse available to use? Most MR systems likely won’t have a mouse!
	pub available: bool,
	/// Position of the mouse relative to the window it’s in! This is the number of pixels from the top left corner of the screen.
	pub pos: Vec2,
	/// How much has the mouse’s position changed in the current frame? Measured in pixels.
	pub pos_change: Vec2,
	/// What’s the current scroll value for the mouse’s scroll wheel? TODO: Units
	pub scroll: f32,
	/// How much has the scroll wheel value changed during this frame? TODO: Units
	pub scroll_change: f32,
}
impl Into<mouse_t> for Mouse {
	fn into(self) -> mouse_t {
		unsafe { std::mem::transmute(self) }
	}
}
impl From<mouse_t> for Mouse {
	fn from(value: mouse_t) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}
/// A collection of system key codes, representing keyboard characters and mouse buttons. Based on VK codes.
#[derive(
	Debug,
	Copy,
	Clone,
	Deserialize_repr,
	Serialize_repr,
	PartialEq,
	Eq,
	IntoPrimitive,
	TryFromPrimitive,
)]
#[repr(u32)]
pub enum Key {
	None = 0,
	MouseLeft = 1,
	MouseRight = 2,
	MouseCenter = 4,
	MouseForward = 5,
	MouseBack = 6,
	Backspace = 8,
	Tab = 9,
	Return = 13,
	Shift = 16,
	Ctrl = 17,
	Alt = 18,
	CapsLock = 20,
	Esc = 27,
	Space = 32,
	End = 35,
	Home = 36,
	Left = 37,
	Right = 39,
	Up = 38,
	Down = 40,
	PageUp = 33,
	PageDown = 34,
	PrintScreen = 42,
	KeyInsert = 45,
	Del = 46,
	Key0 = 48,
	Key1 = 49,
	Key2 = 50,
	Key3 = 51,
	Key4 = 52,
	Key5 = 53,
	Key6 = 54,
	Key7 = 55,
	Key8 = 56,
	Key9 = 57,
	A = 65,
	B = 66,
	C = 67,
	D = 68,
	E = 69,
	F = 70,
	G = 71,
	H = 72,
	I = 73,
	J = 74,
	K = 75,
	L = 76,
	M = 77,
	N = 78,
	O = 79,
	P = 80,
	Q = 81,
	R = 82,
	S = 83,
	T = 84,
	U = 85,
	V = 86,
	W = 87,
	X = 88,
	Y = 89,
	Z = 90,
	Numpad0 = 96,
	Numpad1 = 97,
	Numpad2 = 98,
	Numpad3 = 99,
	Numpad4 = 100,
	Numpad5 = 101,
	Numpad6 = 102,
	Numpad7 = 103,
	Numpad8 = 104,
	Numpad9 = 105,
	F1 = 112,
	F2 = 113,
	F3 = 114,
	F4 = 115,
	F5 = 116,
	F6 = 117,
	F7 = 118,
	F8 = 119,
	F9 = 120,
	F10 = 121,
	F11 = 122,
	F12 = 123,
	Comma = 188,
	Period = 190,
	SlashFwd = 191,
	SlashBack = 220,
	Semicolon = 186,
	Apostrophe = 222,
	BracketOpen = 219,
	BracketClose = 221,
	Minus = 189,
	Equals = 187,
	Backtick = 192,
	LCmd = 91,
	RCmd = 92,
	Multiply = 106,
	Add = 107,
	Subtract = 109,
	Decimal = 110,
	Divide = 111,
}

/// A settings flag that lets you describe the behavior of how StereoKit will refresh data about the world mesh, if applicable. This is used with World.RefreshType.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum WorldRefresh {
	/// Refreshing occurs when the user leaves the area that was most recently scanned. This area is a sphere that is 0.5 of the World.RefreshRadius.
	Area = 0,
	/// Refreshing happens at timer intervals. If an update doesn’t happen in time, the next update will happen as soon as possible. The timer interval is configurable via World.RefreshInterval.
	Timer = 1,
}

/// This describes what technology is being used to power StereoKit’s XR backend.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum BackendXrType {
	/// StereoKit is not using an XR backend of any sort. That means the application is flatscreen and has the simulator disabled.
	None = 0,
	/// StereoKit is using the flatscreen XR simulator. Inputs are emulated, and some advanced XR functionality may not be available.
	Simulator = 1,
	/// StereoKit is currently powered by OpenXR! This means we’re running on a real XR device. Not all OpenXR runtimes provide the same functionality, but we will have access to more fun stuff :)
	OpenXr = 2,
	/// StereoKit is running in a browser, and is using WebXR!
	WebXr = 3,
}

/// This describes the platform that StereoKit is running on.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum BackendPlatform {
	/// This is running as a Windows app using the Win32 APIs.
	Win32 = 0,
	/// This is running as a Windows app using the UWP APIs.
	Uwp = 1,
	/// This is running as a Linux app.
	Linux = 2,
	/// This is running as an Android app.
	Android = 3,
	/// This is running in a browser.
	Web = 4,
}

/// This describes the graphics API thatStereoKit is using for rendering.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum BackendGraphics {
	None = 0,
	D3D11 = 1,
	OpenGlGlx = 2,
	OpenGlWgl = 3,
	OpenGlesEgl = 4,
	WebGl = 5,
}

pub type OpenXrHandle = openxr_handle_t;

/// The log tool will write to the console with annotations for console colors, which helps with readability, but isn’t always supported. These are the options available for configuring those colors.
#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum LogColors {
	/// Use console coloring annotations
	Ansi = 0,
	/// Scrape out any color annotations, so logs are all completely in plain text.
	None = 1,
}

#[derive(Debug, Copy, Clone, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum AssetType {
	None = 0,
	Mesh = 1,
	Tex = 2,
	Shader = 3,
	Material = 4,
	Model = 5,
	Font = 6,
	Sprite = 7,
	Sound = 8,
	Solid = 9,
}

macro_rules! static_material {
    ($id: literal, $name: ident) => {
        concat_idents::concat_idents!(struct_name = Material, $name {
            #[allow(non_camel_case_types)]
            #[allow(dead_code)]
            #[derive(Copy, Clone)]
            #[doc(hidden)]
            pub struct struct_name {
                __private_field: (),
            }
            impl Material {
                pub const $name: struct_name = struct_name { __private_field: () };
            }
            impl AsRef<Material> for struct_name {
                fn as_ref(&self) -> &Material {
                    let id = CString::new($id).unwrap();
                    Box::leak(Box::new(Material(NonNull::new( unsafe {
                        stereokit_sys::material_find(id.as_ptr())
                    }).unwrap())))
                }
            }
        });
    }
}
static_material!("default/material", DEFAULT);
static_material!("default/material_pbr", PBR);
static_material!("default/material_pbr_clip", PBR_CLIP);
static_material!("default/material_unlit", UNLIT);
static_material!("default/material_unlit_clip", UNLIT_CLIP);
static_material!("default/material_equirect", EQUIRECT);
static_material!("default/material_font", FONT);
static_material!("default/material_hand", HAND);
static_material!("default/material_ui", UI);
static_material!("default/material_ui_box", UI_BOX);
static_material!("default/material_ui_quadrant", UI_QUADRANT);

macro_rules! static_tex {
    ($id: literal, $name: ident) => {
        concat_idents::concat_idents!(struct_name = Tex, $name {
            #[allow(non_camel_case_types)]
            #[allow(dead_code)]
            #[derive(Copy, Clone)]
            #[doc(hidden)]
            pub struct struct_name {
                __private_field: (),
            }
            impl Tex {
                pub const $name: struct_name = struct_name { __private_field: () };
            }
            impl AsRef<Tex> for struct_name {
                fn as_ref(&self) -> &Tex {
                    let id = CString::new($id).unwrap();
                    Box::leak(Box::new(Tex(NonNull::new( unsafe {
                        stereokit_sys::tex_find(id.as_ptr())
                    }).unwrap())))
                }
            }
        });
    }
}

static_tex!("default/tex", DEFAULT);
static_tex!("default/tex_black", BLACK);
static_tex!("default/tex_gray", GRAY);
static_tex!("default/tex_flat", FLAT);
static_tex!("default/tex_rough", ROUGH);
static_tex!("default/tex_devtex", DEV_TEX);
static_tex!("default/tex_error", ERROR);
static_tex!("default/cubemap", CUBEMAP);

macro_rules! static_font {
    ($id: literal, $name: ident) => {
        concat_idents::concat_idents!(struct_name = Font, $name {
            #[allow(non_camel_case_types)]
            #[allow(dead_code)]
            #[derive(Copy, Clone)]
            #[doc(hidden)]
            pub struct struct_name {
                __private_field: (),
            }
            impl Font {
                pub const $name: struct_name = struct_name { __private_field: () };
            }
            impl AsRef<Font> for struct_name {
                fn as_ref(&self) -> &Font {
                    let id = CString::new($id).unwrap();
                    Box::leak(Box::new(Font(NonNull::new( unsafe {
                        stereokit_sys::font_find(id.as_ptr())
                    }).unwrap())))
                }
            }
        });
    }
}

static_font!("default/font", DEFAULT);

macro_rules! static_mesh {
    ($id: literal, $name: ident) => {
        concat_idents::concat_idents!(struct_name = Mesh, $name {
            #[allow(non_camel_case_types)]
            #[allow(dead_code)]
            #[derive(Copy, Clone)]
            #[doc(hidden)]
            pub struct struct_name {
                __private_field: (),
            }
            impl Mesh {
                pub const $name: struct_name = struct_name { __private_field: () };
            }
            impl AsRef<Mesh> for struct_name {
                fn as_ref(&self) -> &Mesh {
                    let id = CString::new($id).unwrap();
                    Box::leak(Box::new(Mesh(NonNull::new( unsafe {
                        stereokit_sys::mesh_find(id.as_ptr())
                    }).unwrap())))
                }
            }
        });
    }
}

static_mesh!("default/mesh_quad", QUAD);
static_mesh!("default/mesh_screen_quad", SCREEN_QUAD);
static_mesh!("default/mesh_cube", CUBE);
static_mesh!("default/mesh_sphere", SPHERE);
static_mesh!("default/mesh_lefthand", LEFT_HAND);
static_mesh!("default/mesh_righthand", RIGHT_HAND);
static_mesh!("default/mesh_ui_button", UI_BUTTON);

macro_rules! static_shader {
    ($id: literal, $name: ident) => {
        concat_idents::concat_idents!(struct_name = Shader, $name {
            #[allow(non_camel_case_types)]
            #[allow(dead_code)]
            #[derive(Copy, Clone)]
            #[doc(hidden)]
            pub struct struct_name {
                __private_field: (),
            }
            impl Shader {
                pub const $name: struct_name = struct_name { __private_field: () };
            }
            impl AsRef<Shader> for struct_name {
                fn as_ref(&self) -> &Shader {
                    let id = CString::new($id).unwrap();
                    Box::leak(Box::new(Shader(NonNull::new( unsafe {
                        stereokit_sys::shader_find(id.as_ptr())
                    }).unwrap())))
                }
            }
        });
    }
}

static_shader!("default/shader", DEFAULT);
static_shader!("default/shader_blit", BLIT);
static_shader!("default/shader_pbr", PBR);
static_shader!("default/shader_pbr_clip", PRB_CLIP);
static_shader!("default/shader_font", FONT);
static_shader!("default/shader_ui", UI);
static_shader!("default/shader_ui_box", UI_BOX);
static_shader!("default/shader_ui_quadrant", UI_QUADRANT);
static_shader!("default/shader_sky", SKY);
static_shader!("default/shader_lines", LINES);

macro_rules! static_sound {
    ($id: literal, $name: ident) => {
        concat_idents::concat_idents!(struct_name = Sound, $name {
            #[allow(non_camel_case_types)]
            #[allow(dead_code)]
            #[derive(Copy, Clone)]
            #[doc(hidden)]
            pub struct struct_name {
                __private_field: (),
            }
            impl Sound {
                pub const $name: struct_name = struct_name { __private_field: () };
            }
            impl AsRef<Sound> for struct_name {
                fn as_ref(&self) -> &Sound {
                    let id = CString::new($id).unwrap();
                    Box::leak(Box::new(Sound(NonNull::new( unsafe {
                        stereokit_sys::sound_find(id.as_ptr())
                    }).unwrap())))
                }
            }
        });
    }
}

static_sound!("default/sound_click", CLICK);
static_sound!("default/sound_unclick", UNCLICK);
static_sound!("default/sound_grab", GRAB);
static_sound!("default/sound_ungrab", UNGRAB);

#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum WindowType {
	Empty = 1,
	Head = 2,
	Body = 4,
	Normal = 6,
}

#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr, PartialEq, Eq)]
#[repr(u32)]
pub enum MoveType {
	Exact = 0,
	FaceUser = 1,
	PosOnly = 2,
	None = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCut {
	Left = 0,
	Right = 1,
	Top = 2,
	Bottom = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiColor {
	Primary = 0,
	Background = 1,
	Common = 2,
	Complement = 3,
	Text = 4,
	Max = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiBtnLayout {
	None = 0,
	Left = 1,
	Right = 2,
	Centre = 3,
	CentreNoText = 4,
}

/// All stereokit functions that *must* only be done in the render loop
pub trait StereoKitDraw: StereoKitSingleThread {
	/// Adds a mesh to the render queue for this frame! If the Hierarchy has a transform on it, that transform is combined with the Matrix provided here.
	fn mesh_draw(
		&self,
		mesh: impl AsRef<Mesh>,
		material: impl AsRef<Material>,
		transform: impl Into<Mat4>,
		color_linear: Color128,
		layer: RenderLayer,
	) {
		unsafe {
			stereokit_sys::mesh_draw(
				mesh.as_ref().0.as_ptr(),
				material.as_ref().0.as_ptr(),
				transform.into().into(),
				color_linear,
				layer.bits as IntegerType,
			)
		}
	}

	/// Adds this Model to the render queue for this frame! If the Hierarchy has a transform on it, that transform is combined with the Matrix provided here.
	fn model_draw(
		&self,
		model: impl AsRef<Model>,
		transform: impl Into<Mat4>,
		color_linear: Color128,
		layer: RenderLayer,
	) {
		unsafe {
			stereokit_sys::model_draw(
				model.as_ref().0.as_ptr(),
				transform.into().into(),
				color_linear,
				layer.bits as IntegerType,
			)
		}
	}

	/// Adds a line to the environment for the current frame.
	fn line_add(
		&self,
		start: impl Into<Vec3>,
		end: impl Into<Vec3>,
		color_start: Color32,
		color_end: Color32,
		thickness: f32,
	) {
		unsafe {
			stereokit_sys::line_add(
				start.into().into(),
				end.into().into(),
				color_start,
				color_end,
				thickness,
			)
		}
	}

	fn line_addv(&self, start: LinePoint, end: LinePoint) {
		unsafe { stereokit_sys::line_addv(start.into(), end.into()) }
	}

	/// Displays an RGB/XYZ axis widget at the pose! Each line is extended along the positive direction of each axis, so the red line is +X, green is +Y, and blue is +Z. A white line is drawn along -Z to indicate the Forward vector of the pose (-Z is forward in StereoKit).
	fn line_add_axis(&self, pose: Pose, size: f32) {
		unsafe {
			stereokit_sys::line_add_axis(pose.into(), size);
		}
	}

	fn line_add_list(&self, points: &[Vec3], color: Color32, thickness: f32) {
		unsafe {
			stereokit_sys::line_add_list(
				std::mem::transmute(points.as_ptr()),
				points.len() as i32,
				color,
				thickness,
			)
		}
	}
	fn line_add_listv(&self, points: &[LinePoint]) {
		unsafe {
			stereokit_sys::line_add_listv(std::mem::transmute(points.as_ptr()), points.len() as i32)
		}
	}

	fn render_global_texture(&self, register_slot: i32, texture: impl AsRef<Tex>) {
		unsafe { stereokit_sys::render_global_texture(register_slot, texture.as_ref().0.as_ptr()) }
	}

	/// Adds a mesh to the render queue for this frame! If the Hierarchy has a transform on it, that transform is combined with the Matrix provided here.
	fn render_add_mesh(
		&self,
		mesh: impl AsRef<Mesh>,
		material: impl AsRef<Material>,
		transform: impl Into<Mat4>,
		color_linear: Color128,
		layer: RenderLayer,
	) {
		let transform = transform.into().into();
		unsafe {
			stereokit_sys::render_add_mesh(
				mesh.as_ref().0.as_ptr(),
				material.as_ref().0.as_ptr(),
				&transform,
				color_linear,
				layer.bits as IntegerType,
			)
		}
	}

	fn render_add_model(
		&self,
		model: impl AsRef<Model>,
		transform: impl Into<Mat4>,
		color_linear: Color128,
		layer: RenderLayer,
	) {
		let transform = transform.into().into();
		unsafe {
			stereokit_sys::render_add_model(
				model.as_ref().0.as_ptr(),
				&transform,
				color_linear,
				layer.bits as IntegerType,
			)
		}
	}

	/// Renders a Material onto a rendertarget texture! StereoKit uses a 4 vert quad stretched over the surface of the texture, and renders the material onto it to the texture.
	fn render_blit(&self, to_rendertarget: impl AsRef<Tex>, material: impl AsRef<Material>) {
		unsafe {
			stereokit_sys::render_blit(
				to_rendertarget.as_ref().0.as_ptr(),
				material.as_ref().0.as_ptr(),
			)
		}
	}

	/// Schedules a screenshot for the end of the frame! The view will be rendered from the given position at the given point, with a resolution the same size as the screen’s surface. It’ll be saved as a .jpg file at the filename provided.
	fn render_screenshot(
		&self,
		file: impl AsRef<str>,
		from_viewpt: impl Into<Vec3>,
		at: impl Into<Vec3>,
		width: i32,
		height: i32,
		field_of_view_degrees: f32,
	) {
		let file = CString::new(file.as_ref()).unwrap();
		unsafe {
			stereokit_sys::render_screenshot(
				file.as_ptr(),
				from_viewpt.into().into(),
				at.into().into(),
				width,
				height,
				field_of_view_degrees,
			)
		}
	}

	/// This renders the current scene to the indicated rendertarget texture, from the specified viewpoint. This call enqueues a render that occurs immediately before the screen itself is rendered.
	fn render_to(
		&self,
		to_rendertarget: impl AsRef<Tex>,
		camera: impl Into<Mat4>,
		projection: impl Into<Mat4>,
		layer_filter: RenderLayer,
		clear: RenderClear,
		viewport: Rect,
	) {
		let camera = camera.into().into();
		let projection = projection.into().into();
		unsafe {
			stereokit_sys::render_to(
				to_rendertarget.as_ref().0.as_ptr(),
				&camera,
				&projection,
				layer_filter.bits as IntegerType,
				clear as IntegerType,
				viewport,
			)
		}
	}

	fn render_material_to(
		&self,
		to_rendertarget: impl AsRef<Tex>,
		override_material: impl AsRef<Material>,
		camera: impl Into<Mat4>,
		projection: impl Into<Mat4>,
		layer_filter: RenderLayer,
		clear: RenderClear,
		viewport: Rect,
	) {
		let camera = camera.into().into();
		let projection = projection.into().into();
		unsafe {
			stereokit_sys::render_material_to(
				to_rendertarget.as_ref().0.as_ptr(),
				override_material.as_ref().0.as_ptr(),
				&camera,
				&projection,
				layer_filter.bits as IntegerType,
				clear as IntegerType,
				viewport,
			)
		}
	}

	unsafe fn render_get_device(
		&self,
		device: *mut *mut std::os::raw::c_void,
		context: *mut *mut std::os::raw::c_void,
	) {
		stereokit_sys::render_get_device(device, context)
	}
}
/// All stereokit based functions that *must* be done in a single thread
pub trait StereoKitSingleThread: StereoKitMultiThread {
	/// Pushes a transform Matrix onto the stack, and combines it with the Matrix below it. Any draw operation’s Matrix will now be combined with this Matrix to make it relative to the current hierarchy. Use Hierarchy.Pop to remove it from the Hierarchy stack! All Push calls must have an accompanying Pop call.
	fn hierarchy_push(&self, transform: impl Into<Mat4>) {
		let transform = transform.into().into();
		unsafe { stereokit_sys::hierarchy_push(&transform) }
	}

	/// Removes the top Matrix from the stack!
	fn hierarchy_pop(&self) {
		unsafe {
			stereokit_sys::hierarchy_pop();
		}
	}

	/// This is enabled by default. Disabling this will cause any draw call to ignore any Matrices that are on the Hierarchy stack.
	fn hierarchy_set_enabled(&self, enabled: bool) {
		unsafe { stereokit_sys::hierarchy_set_enabled(enabled as bool32_t) }
	}

	/// This is enabled by default. Disabling this will cause any draw call to ignore any Matrices that are on the Hierarchy stack.
	fn hierarchy_is_enabled(&self) -> bool {
		unsafe { stereokit_sys::hierarchy_is_enabled() != 0 }
	}

	fn hierarchy_to_world(&self) -> Mat4 {
		unsafe { *stereokit_sys::hierarchy_to_world() }.into()
	}

	fn hierarchy_to_local(&self) -> Mat4 {
		unsafe { *stereokit_sys::hierarchy_to_local() }.into()
	}

	/// Converts a world space point into the local space of the current Hierarchy stack!
	fn hierarchy_to_local_point(&self, world_point: impl Into<Vec3>) -> Vec3 {
		let world_point = world_point.into().into();
		unsafe { stereokit_sys::hierarchy_to_local_point(&world_point) }.into()
	}

	/// Converts a world space direction into the local space of the current Hierarchy stack! This excludes the translation component normally applied to vectors, so it’s still a valid direction.
	fn hierarchy_to_local_direction(&self, world_dir: impl Into<Vec3>) -> Vec3 {
		let world_dir = world_dir.into().into();
		unsafe { stereokit_sys::hierarchy_to_local_direction(&world_dir) }.into()
	}

	fn hierarchy_to_local_rotation(&self, world_orientation: impl Into<Quat>) -> Quat {
		let q = world_orientation.into();
		let quat = quat {
			x: q.x,
			y: q.y,
			z: q.z,
			w: q.w,
		};
		let q2 = unsafe { stereokit_sys::hierarchy_to_local_rotation(&quat) };
		Quat::from_xyzw(q2.x, q2.y, q2.z, q2.w)
	}

	fn hierarchy_to_local_pose(&self, world_pose: Pose) -> Pose {
		let pose = world_pose.into();
		unsafe { stereokit_sys::hierarchy_to_local_pose(&pose) }.into()
	}

	fn hierarchy_to_world_point(&self, local_pt: impl Into<Vec3>) -> Vec3 {
		let local_pt = local_pt.into().into();
		unsafe { stereokit_sys::hierarchy_to_world_point(&local_pt) }.into()
	}

	/// Converts a local direction relative to the current hierarchy stack into world space! This excludes the translation component normally applied to vectors, so it’s still a valid direction.
	fn hierarchy_to_world_direction(&self, local_dir: impl Into<Vec3>) -> Vec3 {
		let local_dir = local_dir.into().into();
		unsafe { stereokit_sys::hierarchy_to_world_direction(&local_dir) }.into()
	}

	fn hierarchy_to_world_rotation(&self, local_orientation: impl Into<Quat>) -> Quat {
		let q = local_orientation.into();
		let l1 = quat {
			x: q.x,
			y: q.y,
			z: q.z,
			w: q.w,
		};
		let q2 = unsafe { stereokit_sys::hierarchy_to_world_rotation(&l1) };
		Quat::from_xyzw(q2.x, q2.y, q2.z, q2.w)
	}

	fn hierarchy_to_world_pose(&self, local_pose: Pose) -> Pose {
		let pose = local_pose.into();
		unsafe { stereokit_sys::hierarchy_to_world_pose(&pose) }.into()
	}
}
pub trait StereoKitMultiThread {
	/// Shuts down all StereoKit initialized systems. Release your own StereoKit created assets before calling this.
	fn shutdown(&self) {
		unsafe {
			stereokit_sys::sk_shutdown();
		}
	}
	/// Lets StereoKit know it should quit! It’ll finish the current frame, and after that Step will return that it wants to exit.
	fn quit(&self) {
		unsafe {
			stereokit_sys::sk_quit();
		}
	}
	/// Since we can fallback to a different DisplayMode, this lets you check to see which Runtime was successfully initialized.
	fn active_display_mode(&self) -> DisplayMode {
		unsafe { stereokit_sys::sk_active_display_mode() }.into()
	}
	/// This is a copy of the settings that StereoKit was initialized with, so you can refer back to them a little easier.
	/// These are read only, and keep in mind that some settings are only requests!
	/// Check SK.System and other properties for the current state of StereoKit.
	fn get_settings(&self) -> Settings {
		unsafe { stereokit_sys::sk_get_settings() }.into()
	}

	/// This structure contains information about the current system and its capabilities.
	/// There’s a lot of different MR devices, so it’s nice to have code for systems with particular characteristics!
	fn system_info(&self) -> SystemInfo {
		unsafe { stereokit_sys::sk_system_info() }.into()
	}

	/// Human-readable version name embedded in the StereoKitC DLL.
	fn version_name(&self) -> &str {
		unsafe {
			CStr::from_ptr(stereokit_sys::sk_version_name())
				.to_str()
				.unwrap()
		}
	}
	/// An integer version Id! This is defined using a hex value with this format: 0xMMMMiiiiPPPPrrrr in order of Major.mInor.Patch.pre-Release
	fn version_id(&self) -> u64 {
		unsafe { stereokit_sys::sk_version_id() }
	}

	/// This tells about the app’s current focus state, whether it’s active and receiving input, or if it’s backgrounded or hidden. This can be important since apps may still run and render when unfocused, as the app may still be visible behind the app that does have focus.
	fn app_focus(&self) -> AppFocus {
		unsafe { stereokit_sys::sk_app_focus() }.into()
	}

	/// What type of display is this? Most XR headsets will report
	/// stereo, but the Simulator will report flatscreen.
	fn device_display_get_type(&self) -> DisplayType {
		unsafe { stereokit_sys::device_display_get_type() }.into()
	}

	/// Allows you to get the current blend mode of the device!
	fn device_display_get_blend(&self) -> DisplayBlend {
		unsafe { stereokit_sys::device_display_get_blend() }.into()
	}

	/// Allows you to set the current blend mode of the device!
	/// Setting this may not succeed if the blend mode is not valid.
	fn device_display_set_blend(&self, blend: DisplayBlend) -> bool {
		unsafe { stereokit_sys::device_display_set_blend(blend.into()) != 0 }
	}

	/// Tells if a particular blend mode is valid on this device.
	/// Some devices may be capable of more then one blend mode.
	fn device_display_valid_blend(&self, blend: DisplayBlend) -> bool {
		unsafe { stereokit_sys::device_display_valid_blend(blend.into()) != 0 }
	}

	fn device_display_get_refresh_rate(&self) -> f32 {
		unsafe { stereokit_sys::device_display_get_refresh_rate() }
	}

	fn device_display_get_width(&self) -> i32 {
		unsafe { stereokit_sys::device_display_get_width() }
	}

	fn device_display_get_height(&self) -> i32 {
		unsafe { stereokit_sys::device_display_get_height() }
	}

	fn device_display_get_fov(&self) -> FovInfo {
		unsafe { stereokit_sys::device_display_get_fov() }
	}

	/// The tracking capabilities of this device! Is it 3DoF,
	/// rotation only? Or is it 6DoF, with positional tracking as well?
	/// Maybe it can't track at all!
	fn device_get_tracking(&self) -> DeviceTracking {
		unsafe { stereokit_sys::device_get_tracking() }.into()
	}

	/// This is the name of the active device! From OpenXR, this
	/// is the same as systemName from XrSystemProperties. The simulator
	/// will say "Simulator".
	fn device_get_name(&self) -> &str {
		unsafe {
			CStr::from_ptr(stereokit_sys::device_get_name())
				.to_str()
				.unwrap()
		}
	}

	/// The reported name of the GPU, this will differ between D3D
	/// and GL.
	fn device_get_gpu(&self) -> &str {
		unsafe {
			CStr::from_ptr(stereokit_sys::device_get_gpu())
				.to_str()
				.unwrap()
		}
	}

	/// Does the device we're on have eye tracking support present
	/// for input purposes? This is _not_ an indicator that the user has
	/// given the application permission to access this information. See
	/// `Input.Gaze` for how to use this data.
	fn device_has_eye_gaze(&self) -> bool {
		unsafe { stereokit_sys::device_has_eye_gaze() != 0 }
	}

	/// Tells if the device is capable of tracking hands. This
	/// does not tell if the user is actually using their hands for input,
	/// merely that it's possible to!
	fn device_has_hand_tracking(&self) -> bool {
		unsafe { stereokit_sys::device_has_hand_tracking() != 0 }
	}

	fn time_get_raw(&self) -> f64 {
		unsafe { stereokit_sys::time_get_raw() }
	}

	fn time_get_f32_unscaled(&self) -> f32 {
		unsafe { stereokit_sys::time_getf_unscaled() }
	}

	fn time_get_unscaled(&self) -> f64 {
		unsafe { stereokit_sys::time_get_unscaled() }
	}

	fn time_get_f32(&self) -> f32 {
		unsafe { stereokit_sys::time_getf() }
	}

	fn time_get(&self) -> f64 {
		unsafe { stereokit_sys::time_get() }
	}

	fn time_elapsed_f32_unscaled(&self) -> f32 {
		unsafe { stereokit_sys::time_elapsedf_unscaled() }
	}

	fn time_elapsed_unscaled(&self) -> f64 {
		unsafe { stereokit_sys::time_elapsed_unscaled() }
	}

	fn time_total_raw(&self) -> f64 {
		unsafe { stereokit_sys::time_total_raw() }
	}

	fn time_total_f32_unscaled(&self) -> f32 {
		unsafe { stereokit_sys::time_totalf_unscaled() }
	}

	fn time_total_unscaled(&self) -> f64 {
		unsafe { stereokit_sys::time_total_unscaled() }
	}

	fn time_total_f32(&self) -> f32 {
		unsafe { stereokit_sys::time_totalf() }
	}

	fn time_total(&self) -> f64 {
		unsafe { stereokit_sys::time_total() }
	}

	fn time_step_f32_unscaled(&self) -> f32 {
		unsafe { stereokit_sys::time_stepf_unscaled() }
	}

	fn time_step_unscaled(&self) -> f64 {
		unsafe { stereokit_sys::time_step_unscaled() }
	}

	fn time_step_f32(&self) -> f32 {
		unsafe { stereokit_sys::time_stepf() }
	}

	fn time_step(&self) -> f64 {
		unsafe { stereokit_sys::time_step() }
	}

	fn time_scale(&self, scale: f64) {
		unsafe { stereokit_sys::time_scale(scale) }
	}

	/// This allows you to override the application time! The application will progress from this time using the current timescale.
	fn time_set_time(&self, total_seconds: f64, frame_elapsed_seconds: f64) {
		unsafe { stereokit_sys::time_set_time(total_seconds, frame_elapsed_seconds) }
	}

	/// Finds the Mesh with the matching id, and returns a reference to it. If no Mesh it found, it returns Error
	fn mesh_find<S: Into<String> + Clone>(&self, name: S) -> SkResult<Mesh> {
		let c_str = std::ffi::CString::new(name.clone().into())
			.map_err(|_| StereoKitError::MeshCString(name.clone().into()))?;
		Ok(Mesh(
			NonNull::new(unsafe { stereokit_sys::mesh_find(c_str.as_ptr()) })
				.ok_or(StereoKitError::MeshFind(name.into()))?,
		))
	}

	/// Creates an empty Mesh asset. Use SetVerts and SetInds to add data to it!
	fn mesh_create(&self) -> Mesh {
		Mesh(NonNull::new(unsafe { stereokit_sys::mesh_create() }).unwrap())
	}

	/// Makes a copy of the mesh
	fn mesh_copy<Me: AsRef<Mesh>>(&self, mesh: Me) -> Mesh {
		Mesh(NonNull::new(unsafe { stereokit_sys::mesh_copy(mesh.as_ref().0.as_ptr()) }).unwrap())
	}

	fn mesh_set_id<Me: AsRef<Mesh>, S: Into<String>>(&self, mesh: Me, id: S) {
		let c_str = std::ffi::CString::new(id.into()).unwrap();
		unsafe { stereokit_sys::mesh_set_id(mesh.as_ref().0.as_ptr(), c_str.as_ptr()) }
	}

	fn mesh_get_id<Me: AsRef<Mesh>>(&self, mesh: Me) -> &str {
		unsafe { CStr::from_ptr(stereokit_sys::mesh_get_id(mesh.as_ref().0.as_ptr())) }
			.to_str()
			.unwrap()
	}

	fn mesh_addref<Me: AsRef<Mesh>>(&self, mesh: Me) {
		unsafe { stereokit_sys::mesh_addref(mesh.as_ref().0.as_ptr()) }
	}

	/// Releases the asset, automatically called on drop.
	fn mesh_release(&self, _mesh: Mesh) {}

	fn mesh_set_keep_data<Me: AsRef<Mesh>>(&self, mesh: Me, keep_data: bool) {
		unsafe {
			stereokit_sys::mesh_set_keep_data(
				mesh.as_ref().0.as_ptr(),
				keep_data as stereokit_sys::bool32_t,
			);
		}
	}

	fn mesh_get_keep_data<Me: AsRef<Mesh>>(&self, mesh: Me) -> bool {
		unsafe { stereokit_sys::mesh_get_keep_data(mesh.as_ref().0.as_ptr()) != 0 }
	}

	fn mesh_set_data<Me: AsRef<Mesh>>(
		&self,
		mesh: Me,
		vertices: &[Vert],
		arrays: &[u32],
		calculate_bounds: bool,
	) {
		unsafe {
			stereokit_sys::mesh_set_data(
				mesh.as_ref().0.as_ptr(),
				std::mem::transmute(vertices.as_ptr()),
				vertices.len() as i32,
				std::mem::transmute(arrays.as_ptr()),
				arrays.len() as i32,
				calculate_bounds as bool32_t,
			)
		}
	}

	/// Assigns the face indices for this Mesh! Faces are always triangles, there are only ever three indices per face. This function will create a index buffer object on the graphics card right away. If you’re calling this a second time, the buffer will be marked as dynamic and re-allocated. If you’re calling this a third time, the buffer will only re-allocate if the buffer is too small, otherwise it just copies in the data!
	fn mesh_set_verts<Me: AsRef<Mesh>>(&self, mesh: Me, vertices: &[Vert], calculate_bounds: bool) {
		unsafe {
			stereokit_sys::mesh_set_verts(
				mesh.as_ref().0.as_ptr(),
				std::mem::transmute(vertices.as_ptr()),
				vertices.len() as i32,
				calculate_bounds as bool32_t,
			)
		}
	}

	/// GetVerts	This marshalls the Mesh’s vertex data into an array. If KeepData is false, then the Mesh is not storing verts on the CPU, and this information will not be available. Due to the way marshalling works, this is not a cheap function!
	fn mesh_get_verts_ref(&self, mesh: &Mesh) -> &mut [Vert] {
		unsafe {
			let mut verts_pointer = null_mut();
			let mut verts_len = 0;
			stereokit_sys::mesh_get_verts(mesh.0.as_ptr(), &mut verts_pointer, &mut verts_len, 0);
			&mut *slice_from_raw_parts_mut(std::mem::transmute(verts_pointer), verts_len as usize)
		}
	}

	/// GetVerts	This marshalls the Mesh’s vertex data into an array. If KeepData is false, then the Mesh is not storing verts on the CPU, and this information will not be available. Due to the way marshalling works, this is not a cheap function!
	fn mesh_get_verts_copy<Me: AsRef<Mesh>>(&self, mesh: Me) -> Vec<Vert> {
		unsafe {
			let mut verts_pointer = null_mut();
			let mut verts_len = 0;
			stereokit_sys::mesh_get_verts(
				mesh.as_ref().0.as_ptr(),
				&mut verts_pointer,
				&mut verts_len,
				0,
			);
			Vec::from_raw_parts(
				std::mem::transmute(verts_pointer),
				verts_len as usize,
				verts_len as usize,
			)
		}
	}

	/// Assigns the face indices for this Mesh! Faces are always triangles, there are only ever three indices per face. This function will create a index buffer object on the graphics card right away. If you’re calling this a second time, the buffer will be marked as dynamic and re-allocated. If you’re calling this a third time, the buffer will only re-allocate if the buffer is too small, otherwise it just copies in the data!
	fn mesh_set_inds(&self, mesh: &Mesh, inds: &[i32]) {
		unsafe {
			stereokit_sys::mesh_set_inds(
				mesh.0.as_ptr(),
				std::mem::transmute(inds.as_ptr()),
				inds.len() as i32,
			)
		}
	}

	/// This marshalls the Mesh’s index data into an array. If KeepData is false, then the Mesh is not storing indices on the CPU, and this information will not be available. Due to the way marshalling works, this is not a cheap function!
	fn mesh_get_inds_ref(&self, mesh: &Mesh) -> &mut [i32] {
		unsafe {
			let mut inds_ptr = null_mut();
			let mut inds_len = 0;
			stereokit_sys::mesh_get_inds(mesh.0.as_ptr(), &mut inds_ptr, &mut inds_len, 0);
			&mut *slice_from_raw_parts_mut(std::mem::transmute(inds_ptr), inds_len as usize)
		}
	}

	/// This marshalls the Mesh’s index data into an array. If KeepData is false, then the Mesh is not storing indices on the CPU, and this information will not be available. Due to the way marshalling works, this is not a cheap function!
	fn mesh_get_inds_copy<Me: AsRef<Mesh>>(&self, mesh: Me) -> Vec<i32> {
		unsafe {
			let mut inds_ptr = null_mut();
			let mut inds_len = 0;
			stereokit_sys::mesh_get_inds(mesh.as_ref().0.as_ptr(), &mut inds_ptr, &mut inds_len, 1);
			Vec::from_raw_parts(
				std::mem::transmute(inds_ptr),
				inds_len as usize,
				inds_len as usize,
			)
		}
	}

	/// The number of vertices stored in this Mesh! This is available to you regardless of whether or not KeepData is set.
	fn mesh_get_ind_count<Me: AsRef<Mesh>>(&self, mesh: Me) -> i32 {
		unsafe { stereokit_sys::mesh_get_ind_count(mesh.as_ref().0.as_ptr()) }
	}

	/// This is a bounding box that encapsulates the Mesh! It’s used for collision, visibility testing, UI layout, and probably other things. While it’s normally calculated from the mesh vertices, you can also override this to suit your needs.
	fn mesh_set_bounds<Me: AsRef<Mesh>>(&self, mesh: Me, bounds: impl AsRef<Bounds>) {
		unsafe {
			stereokit_sys::mesh_set_bounds(
				mesh.as_ref().0.as_ptr(),
				std::mem::transmute(bounds.as_ref() as *const Bounds),
			);
		}
	}

	/// This is a bounding box that encapsulates the Mesh! It’s used for collision, visibility testing, UI layout, and probably other things. While it’s normally calculated from the mesh vertices, you can also override this to suit your needs.
	fn mesh_get_bounds<Me: AsRef<Mesh>>(&self, mesh: Me) -> Bounds {
		unsafe { stereokit_sys::mesh_get_bounds(mesh.as_ref().0.as_ptr()) }.into()
	}

	fn mesh_has_skin<Me: AsRef<Mesh>>(&self, mesh: Me) -> bool {
		unsafe { stereokit_sys::mesh_has_skin(mesh.as_ref().0.as_ptr()) != 0 }
	}

	fn mesh_set_skin<Me: AsRef<Mesh>>(
		&self,
		mesh: Me,
		bone_ids: &[u16],
		bone_weights: &[Vec4],
		bone_resting_transforms: &[Mat4],
	) {
		let bone_resting_transforms = bone_resting_transforms
			.into_iter()
			.map(|a| a.clone().into())
			.collect::<Vec<_>>();
		unsafe {
			stereokit_sys::mesh_set_skin(
				mesh.as_ref().0.as_ptr(),
				bone_ids.as_ptr(),
				bone_ids.len() as i32,
				std::mem::transmute(bone_weights.as_ptr()),
				bone_weights.len() as i32,
				bone_resting_transforms.as_ptr(),
				bone_resting_transforms.len() as i32,
			)
		}
	}

	fn mesh_update_skin<Me: AsRef<Mesh>>(&self, mesh: Me, bone_transforms: &[Mat4]) {
		let bone_transforms = bone_transforms
			.iter()
			.map(|a| a.clone().into())
			.collect::<Vec<_>>();
		unsafe {
			stereokit_sys::mesh_update_skin(
				mesh.as_ref().0.as_ptr(),
				bone_transforms.as_ptr(),
				bone_transforms.len() as i32,
			)
		}
	}

	fn mesh_ray_intersect<Me: AsRef<Mesh>>(
		&self,
		mesh: Me,
		model_space_ray: Ray,
		cull_mode: CullMode,
	) -> Option<(Ray, u32)> {
		let mut out_ray = Box::new(ray_t { pos: stereokit_sys::vec3 {
			x: 0.0,
			y: 0.0,
			z: 0.0,
		}, dir: stereokit_sys::vec3 {
			x: 0.0,
			y: 0.0,
			z: 0.0,
		} });

		let mut out_inds = 0;
		match unsafe {
			stereokit_sys::mesh_ray_intersect(
				mesh.as_ref().0.as_ptr(),
				model_space_ray.into(),
				out_ray.as_mut() as *mut ray_t,
				&mut out_inds,
				cull_mode as cull_,
			) != 0
		} {
			true => Some(( (*out_ray).into() , out_inds)),
			false => None,
		}
	}

	fn mesh_ray_intersect_bvh<Me: AsRef<Mesh>>(
		&self,
		mesh: Me,
		model_space_ray: Ray,
		cull_mode: CullMode,
	) -> Option<(Ray, u32)> {
		let out_ptr = null_mut();
		let mut out_inds = 0;
		match unsafe {
			stereokit_sys::mesh_ray_intersect_bvh(
				mesh.as_ref().0.as_ptr(),
				model_space_ray.into(),
				out_ptr,
				&mut out_inds,
				cull_mode as cull_,
			) != 0
		} {
			true => Some((unsafe { (*out_ptr).into() }, out_inds)),
			false => None,
		}
	}

	/// Retrieves the vertices associated with a particular triangle on the Mesh.
	fn mesh_get_triangle<Me: AsRef<Mesh>>(
		&self,
		mesh: Me,
		triangle_index: u32,
	) -> Option<[Vert; 3]> {
		let out_a = null_mut();
		let out_b = null_mut();
		let out_c = null_mut();
		unsafe {
			match stereokit_sys::mesh_get_triangle(
				mesh.as_ref().0.as_ptr(),
				triangle_index,
				out_a,
				out_b,
				out_c,
			) != 0
			{
				true => Some([(*out_a).into(), (*out_b).into(), (*out_c).into()]),
				false => None,
			}
		}
	}

	/// GeneratePlane	Generates a plane on the XZ axis facing up that is optionally subdivided, pre-sized to the given dimensions. UV coordinates start at 0,0 at the -X,-Z corer, and go to 1,1 at the +X,+Z corner! NOTE: This generates a completely new Mesh asset on the GPU, and is best done during ‘initialization’ of your app/scene. You may also be interested in using the pre-generated Mesh.Quad asset if it already meets your needs.
	fn mesh_gen_plane(
		&self,
		dimensions: Vec2,
		plane_normal: Vec3,
		plane_top_direction: Vec3,
		subdivisions: i32,
		double_sided: bool,
	) -> Mesh {
		unsafe {
			stereokit_sys::mesh_gen_plane(
				dimensions.into(),
				plane_normal.into(),
				plane_top_direction.into(),
				subdivisions.into(),
				double_sided as bool32_t,
			)
		}
		.into()
	}

	fn mesh_gen_circle(
		&self,
		diameter: f32,
		plane_normal: Vec3,
		plane_top_direction: Vec3,
		spokes: i32,
		double_sided: bool,
	) -> Mesh {
		unsafe {
			stereokit_sys::mesh_gen_circle(
				diameter,
				plane_normal.into(),
				plane_top_direction.into(),
				spokes.into(),
				double_sided as bool32_t,
			)
		}
		.into()
	}

	/// Generates a flat-shaded cube mesh, pre-sized to the given dimensions. UV coordinates are projected flat on each face, 0,0 -> 1,1. NOTE: This generates a completely new Mesh asset on the GPU, and is best done during ‘initialization’ of your app/scene. You may also be interested in using the pre-generated Mesh.Cube asset if it already meets your needs.
	fn mesh_gen_cube(&self, dimensions: impl Into<Vec3>, subdivisions: i32) -> Mesh {
		unsafe { stereokit_sys::mesh_gen_cube(dimensions.into().into(), subdivisions) }.into()
	}

	/// Generates a sphere mesh, pre-sized to the given diameter, created by sphereifying a subdivided cube! UV coordinates are taken from the initial unspherified cube. NOTE: This generates a completely new Mesh asset on the GPU, and is best done during ‘initialization’ of your app/scene. You may also be interested in using the pre-generated Mesh.Sphere asset if it already meets your needs.
	fn mesh_gen_sphere(&self, diameter: f32, subdivisions: i32) -> Mesh {
		unsafe { stereokit_sys::mesh_gen_sphere(diameter, subdivisions) }.into()
	}

	/// Generates a cube mesh with rounded corners, pre-sized to the given dimensions. UV coordinates are 0,0 -> 1,1 on each face, meeting at the middle of the rounded corners. NOTE: This generates a completely new Mesh asset on the GPU, and is best done during ‘initialization’ of your app/scene.
	fn mesh_gen_rounded_cube(&self, dimensions: Vec3, edge_radius: f32, subdivisions: i32) -> Mesh {
		unsafe {
			stereokit_sys::mesh_gen_rounded_cube(dimensions.into(), edge_radius, subdivisions)
		}
		.into()
	}

	/// Generates a cylinder mesh, pre-sized to the given diameter and depth, UV coordinates are from a flattened top view right now. Additional development is needed for making better UVs for the edges. NOTE: This generates a completely new Mesh asset on the GPU, and is best done during ‘initialization’ of your app/scene.
	fn mesh_gen_cylinder(
		&self,
		diameter: f32,
		depth: f32,
		direction: Vec3,
		subdivisions: i32,
	) -> Mesh {
		unsafe { stereokit_sys::mesh_gen_cylinder(diameter, depth, direction.into(), subdivisions) }
			.into()
	}

	fn mesh_gen_cone(&self, diameter: f32, depth: f32, direction: Vec3, subdivisions: i32) -> Mesh {
		unsafe { stereokit_sys::mesh_gen_cone(diameter, depth, direction.into(), subdivisions) }
			.into()
	}

	fn tex_find<S: Into<String> + Clone>(&self, id: S) -> SkResult<Tex> {
		let c_str = CString::new(id.clone().into())
			.map_err(|_| StereoKitError::TexCString(id.clone().into()))?;

		Ok(Tex(NonNull::new(unsafe {
			stereokit_sys::tex_find(c_str.as_ptr())
		})
		.ok_or(StereoKitError::TexFind(id.clone().into()))?))
	}

	/// Sets up an empty texture container! Fill it with data using SetColors next! Creates a default unique asset Id.
	fn tex_create(&self, r#type: TextureType, format: TextureFormat) -> Tex {
		Tex(NonNull::new(unsafe {
			stereokit_sys::tex_create(r#type.bits as IntegerType, format as tex_format_)
		})
		.unwrap())
	}

	//TODO: fn tex_create_color32()
	//TODO: fn tex_create_color128()

	/// Loads an image file stored in memory directly into a texture! Supported formats are: jpg, png, tga, bmp, psd, gif, hdr, pic. Asset Id will be the same as the filename.
	fn tex_create_mem(&self, data: &[u8], srgb_data: bool, priority: i32) -> SkResult<Tex> {
		Ok(Tex(NonNull::new(unsafe {
			stereokit_sys::tex_create_mem(
				data.as_ptr() as *mut std::os::raw::c_void,
				data.len(),
				srgb_data as bool32_t,
				priority,
			)
		})
		.ok_or(StereoKitError::TexMemory)?))
	}

	/// Loads an image file directly into a texture! Supported formats are: jpg, png, tga, bmp, psd, gif, hdr, pic. Asset Id will be the same as the filename.
	fn tex_create_file(
		&self,
		file_utf8: impl AsRef<Path>,
		srgb_data: bool,
		priority: i32,
	) -> SkResult<Tex> {
		let path = file_utf8.as_ref();
		let path_buf = path.to_path_buf();
		let c_str = CString::new(path.to_str().ok_or(StereoKitError::TexFile(
			path_buf.clone(),
			"CString conversion".to_string(),
		))?)
		.map_err(|_| StereoKitError::TexFile(path_buf.clone(), "CString Conversion".to_string()))?;
		Ok(Tex(NonNull::new(unsafe {
			stereokit_sys::tex_create_file(c_str.as_ptr(), srgb_data as bool32_t, priority)
		})
		.ok_or(StereoKitError::TexFile(
			path_buf.clone(),
			"tex_create_file failed".to_string(),
		))?))
	}

	//TODO: tex_create_file_arr

	/// Creates a cubemap texture from a single equirectangular image! You know, the ones that look like an unwrapped globe with the poles all stretched out. It uses some fancy shaders and texture blitting to create 6 faces from the equirectangular image.
	fn tex_create_cubemap_file(
		&self,
		equirectangular_file_utf8: impl AsRef<Path>,
		srgb_data: bool,
		priority: i32,
	) -> SkResult<(SphericalHarmonics, Tex)> {
		let path = equirectangular_file_utf8.as_ref();
		let path_buf = path.to_path_buf();
		let mut sh: spherical_harmonics_t = sh_create(&[]).into();
		let c_str = CString::new(path.to_str().ok_or(StereoKitError::TexFile(
			path_buf.clone(),
			"CString conversion".to_string(),
		))?)
		.map_err(|_| StereoKitError::TexFile(path_buf.clone(), "CString Conversion".to_string()))?;
		let tex = Tex(NonNull::new(unsafe {
			stereokit_sys::tex_create_cubemap_file(
				c_str.as_ptr(),
				srgb_data as bool32_t,
				&mut sh,
				priority,
			)
		})
		.ok_or(StereoKitError::TexFile(
			path_buf.clone(),
			"tex_create_file failed".to_string(),
		))?);

		Ok((sh.into(), tex))
	}

	//TODO: tex_create_cubemap_files

	/// Allows you to set the Id of the texture to a specific Id.
	fn tex_set_id<T: AsRef<Tex>, S: Into<String> + Clone>(&self, tex: T, id: S) {
		let c_str = CString::new(id.into()).unwrap();
		unsafe { stereokit_sys::tex_set_id(tex.as_ref().0.as_ptr(), c_str.as_ptr()) }
	}

	/// Allows you to set the Id of the texture to a specific Id.
	fn tex_get_id<T: AsRef<Tex>>(&self, tex: T) -> &str {
		unsafe {
			CStr::from_ptr(stereokit_sys::tex_get_id(tex.as_ref().0.as_ptr()))
				.to_str()
				.unwrap()
		}
	}

	/// This will override the default fallback texutre that gets used before the Tex has finished loading. This is useful for textures with a specific purpose where the normal fallback texture would appear strange, such as a metal/rough map.
	fn tex_set_fallback<T: AsRef<Tex>>(&self, tex: T, fallback: T) {
		unsafe {
			stereokit_sys::tex_set_fallback(tex.as_ref().0.as_ptr(), fallback.as_ref().0.as_ptr())
		}
	}

	unsafe fn tex_set_surface<T: AsRef<Tex>>(
		&self,
		tex: T,
		native_surface: *mut std::os::raw::c_void,
		r#type: TextureType,
		native_fmt: i64,
		width: i32,
		height: i32,
		surface_count: i32,
		owned: bool,
	) {
		stereokit_sys::tex_set_surface(
			tex.as_ref().0.as_ptr(),
			native_surface,
			r#type.bits as IntegerType,
			native_fmt,
			width,
			height,
			surface_count,
			owned as bool32_t,
		)
	}

	unsafe fn tex_get_surface<T: AsRef<Tex>>(&self, tex: T) -> *mut std::os::raw::c_void {
		stereokit_sys::tex_get_surface(tex.as_ref().0.as_ptr())
	}

	/// increments the reference count, don't use this unless you plan to decrement manually
	unsafe fn tex_addref<T: AsRef<Tex>>(&self, tex: T) {
		unsafe { stereokit_sys::tex_addref(tex.as_ref().0.as_ptr()) }
	}

	/// Textures are loaded asyncronously, so this tells you the current state of this texture! This also can tell if an error occured, and what type of error it may have been.
	fn tex_asset_state<T: AsRef<Tex>>(&self, tex: T) -> AssetState {
		unsafe { std::mem::transmute(stereokit_sys::tex_asset_state(tex.as_ref().0.as_ptr())) }
	}

	//TODO: tex_on_load

	//TODO: tex_on_load_remove

	//TODO: tex_set_colors, need to use width*height checking

	//TODO: tex_set_color_arr

	fn tex_set_mem<T: AsRef<Tex>>(&self, tex: T, data: &[u8], srgb_data: bool, blocking: i32, priority: i32) {
		unsafe {
			stereokit_sys::tex_set_mem(
				tex.as_ref().0.as_ptr(),
				data.as_ptr() as *mut std::os::raw::c_void,
				data.len(),
				srgb_data as bool32_t,
				blocking,
				priority,
			)
		}
	}

	/// Only applicable if this texture is a rendertarget! This creates and attaches a zbuffer surface to the texture for use when rendering to it.
	fn tex_add_zbuffer<T: AsRef<Tex>>(&self, tex: T, format: TextureFormat) -> Tex {
		Tex(NonNull::new(unsafe {
			stereokit_sys::tex_add_zbuffer(tex.as_ref().0.as_ptr(), format as tex_format_)
		})
		.unwrap())
	}

	//TODO: this is wrong, we need to allocate a buffer and insert it
	// it is width * height * sizeof<color_format>
	// fn tex_get_data<T: AsRef<Tex>>(&self, tex: T) -> &mut [u8] {
	//     let ptr = null_mut();
	//     let mut size = 0;
	//     unsafe {
	//         stereokit_sys::tex_get_data(tex.as_ref().0.as_ptr(), ptr, size);
	//         &mut *slice_from_raw_parts_mut(std::mem::transmute(ptr), size)
	//     }
	// }

	//TODO: tex_get_data_mip

	/// This generates a solid color texture of the given dimensions. Can be quite nice for creating placeholder textures! Make sure to match linear/gamma colors with the correct format.
	fn tex_gen_color(
		&self,
		color: Color128,
		width: i32,
		height: i32,
		type_: TextureType,
		format: TextureFormat,
	) -> Tex {
		Tex(NonNull::new(unsafe {
			stereokit_sys::tex_gen_color(
				color,
				width,
				height,
				type_.bits as IntegerType,
				format as tex_format_,
			)
		})
		.unwrap())
	}

	fn tex_gen_particle(
		&self,
		width: i32,
		height: i32,
		roundness: f32,
		gradient_linear: impl AsRef<Gradient>,
	) -> Tex {
		Tex(NonNull::new(unsafe {
			stereokit_sys::tex_gen_particle(
				width,
				height,
				roundness,
				gradient_linear.as_ref().0.as_ptr(),
			)
		})
		.unwrap())
	}

	/// Generates a cubemap texture from a gradient and a direction! These are entirely suitable for skyboxes, which you can set via Renderer.SkyTex.
	fn tex_gen_cubemap(
		&self,
		gradient: impl AsRef<Gradient>,
		gradient_dir: impl Into<Vec3>,
		resolution: i32,
	) -> (SphericalHarmonics, Tex) {
		let sphere_ptr = &mut sh_create(&[]).into();
		let tex = unsafe {
			stereokit_sys::tex_gen_cubemap(
				gradient.as_ref().0.as_ptr(),
				gradient_dir.into().into(),
				resolution,
				sphere_ptr,
			)
		};
		(
			unsafe { std::mem::transmute(*sphere_ptr) },
			Tex(NonNull::new(tex).unwrap()),
		)
	}

	fn tex_gen_cubemap_sh(
		&self,
		lookup: &SphericalHarmonics,
		face_size: i32,
		light_spot_size_pct: f32,
		light_spot_intensity: f32,
	) -> Tex {
		Tex(NonNull::new(unsafe {
			stereokit_sys::tex_gen_cubemap_sh(
				std::mem::transmute(lookup as *const SphericalHarmonics),
				face_size,
				light_spot_size_pct,
				light_spot_intensity,
			)
		})
		.unwrap())
	}

	/// The StereoKit format this texture was initialized with. This will be a blocking call if AssetState is less than LoadedMeta.
	fn tex_get_format<T: AsRef<Tex>>(&self, tex: T) -> TextureFormat {
		unsafe { std::mem::transmute(stereokit_sys::tex_get_format(tex.as_ref().0.as_ptr())) }
	}

	/// The width of the texture, in pixels. This will be a blocking call if AssetState is less than LoadedMeta.
	fn tex_get_width<T: AsRef<Tex>>(&self, tex: T) -> i32 {
		unsafe { stereokit_sys::tex_get_width(tex.as_ref().0.as_ptr()) }
	}

	/// The height of the texture, in pixels. This will be a blocking call if AssetState is less than LoadedMeta.
	fn tex_get_height<T: AsRef<Tex>>(&self, tex: T) -> i32 {
		unsafe { stereokit_sys::tex_get_height(tex.as_ref().0.as_ptr()) }
	}

	/// When sampling a texture that’s stretched, or shrunk beyond its screen size, how do we handle figuring out which color to grab from the texture? Default is Linear.
	fn tex_set_sample<T: AsRef<Tex>>(&self, tex: T, sample: TextureSample) {
		unsafe { stereokit_sys::tex_set_sample(tex.as_ref().0.as_ptr(), sample as tex_sample_) }
	}

	/// When sampling a texture that’s stretched, or shrunk beyond its screen size, how do we handle figuring out which color to grab from the texture? Default is Linear.
	fn tex_get_sample<T: AsRef<Tex>>(&self, tex: T) -> TextureSample {
		unsafe { std::mem::transmute(stereokit_sys::tex_get_sample(tex.as_ref().0.as_ptr())) }
	}

	/// When looking at a UV texture coordinate on this texture, how do we handle values larger than 1, or less than zero? Do we Wrap to the other side? Clamp it between 0-1, or just keep Mirroring back and forth? Wrap is the default.
	fn tex_set_address<T: AsRef<Tex>>(&self, tex: T, address_mode: TextureAddress) {
		unsafe {
			stereokit_sys::tex_set_address(tex.as_ref().0.as_ptr(), address_mode as tex_address_)
		}
	}

	/// When looking at a UV texture coordinate on this texture, how do we handle values larger than 1, or less than zero? Do we Wrap to the other side? Clamp it between 0-1, or just keep Mirroring back and forth? Wrap is the default.
	fn tex_get_address<T: AsRef<Tex>>(&self, tex: T) -> TextureAddress {
		unsafe { std::mem::transmute(stereokit_sys::tex_get_address(tex.as_ref().0.as_ptr())) }
	}

	/// When SampleMode is set to Anisotropic, this is the number of samples the GPU takes to figure out the correct color. Default is 4, and 16 is pretty high.
	fn tex_set_anisotropy<T: AsRef<Tex>>(&self, tex: T, anisotropy_level: i32) {
		unsafe { stereokit_sys::tex_set_anisotropy(tex.as_ref().0.as_ptr(), anisotropy_level) }
	}

	/// When SampleMode is set to Anisotropic, this is the number of samples the GPU takes to figure out the correct color. Default is 4, and 16 is pretty high.
	fn tex_get_anisotropy<T: AsRef<Tex>>(&self, tex: T) -> i32 {
		unsafe { stereokit_sys::tex_get_anisotropy(tex.as_ref().0.as_ptr()) }
	}

	fn tex_get_mips<T: AsRef<Tex>>(&self, tex: T) -> i32 {
		unsafe { stereokit_sys::tex_get_mips(tex.as_ref().0.as_ptr()) }
	}

	/// This is the texture that all Tex objects will fall back to by default if they are still loading. Assigning a texture here that isn’t fully loaded will cause the app to block until it is loaded.
	fn tex_set_loading_fallback<T: AsRef<Tex>>(&self, tex: T) {
		unsafe { stereokit_sys::tex_set_loading_fallback(tex.as_ref().0.as_ptr()) }
	}

	/// This is the texture that all Tex objects with errors will fall back to. Assigning a texture here that isn’t fully loaded will cause the app to block until it is loaded.
	fn tex_set_error_fallback<T: AsRef<Tex>>(&self, error_texture: T) {
		unsafe { stereokit_sys::tex_set_error_fallback(error_texture.as_ref().0.as_ptr()) }
	}

	fn tex_get_cubemap_lighting<T: AsRef<Tex>>(&self, cubemap_texture: T) -> SphericalHarmonics {
		unsafe {
			std::mem::transmute(stereokit_sys::tex_get_cubemap_lighting(
				cubemap_texture.as_ref().0.as_ptr(),
			))
		}
	}

	/// Searches the asset list for a font with the given Id, returning Err if none is found.
	fn font_find<S: Into<String> + Clone>(&self, id: S) -> SkResult<Font> {
		let c_str = CString::new(id.clone().into()).map_err(|_| {
			StereoKitError::FontFind(id.clone().into(), "CString conversion error".to_string())
		})?;
		Ok(Font(
			NonNull::new(unsafe { stereokit_sys::font_find(c_str.as_ptr()) }).ok_or(
				StereoKitError::FontFind(id.clone().into(), "font_find failed".to_string()),
			)?,
		))
	}

	/// Loads a font and creates a font asset from it.
	fn font_create(&self, file_utf8: impl AsRef<Path>) -> SkResult<Font> {
		let path = file_utf8.as_ref();
		let path_buf = path.to_path_buf();
		let c_str = CString::new(path_buf.clone().to_str().ok_or(StereoKitError::FontFile(
			path_buf.clone(),
			"CString conversion".to_string(),
		))?)
		.map_err(|_| {
			StereoKitError::FontFile(path_buf.clone(), "CString conversion".to_string())
		})?;
		Ok(Font(
			NonNull::new(unsafe { stereokit_sys::font_create(c_str.as_ptr()) }).ok_or(
				StereoKitError::FontFile(path_buf.clone(), "font_create failed".to_string()),
			)?,
		))
	}

	fn font_set_id<S: Into<String> + Clone>(&self, font: impl AsRef<Font>, id: S) {
		let c_str = CString::new(id.into()).unwrap();
		unsafe { stereokit_sys::font_set_id(font.as_ref().0.as_ptr(), c_str.as_ptr()) }
	}

	fn font_get_id(&self, font: impl AsRef<Font>) -> &str {
		unsafe { CStr::from_ptr(stereokit_sys::font_get_id(font.as_ref().0.as_ptr())) }
			.to_str()
			.unwrap()
	}

	unsafe fn font_addref(&self, font: impl AsRef<Font>) {
		unsafe { stereokit_sys::font_addref(font.as_ref().0.as_ptr()) }
	}

	fn font_release(&self, _font: Font) {}

	/// Looks for a Shader asset that’s already loaded, matching the given id! Unless the id has been set manually, the id will be the same as the shader’s name provided in the metadata.
	fn shader_find<S: Into<String> + Clone>(&self, id: S) -> SkResult<Shader> {
		let c_str = CString::new(id.clone().into()).map_err(|_| {
			StereoKitError::ShaderFind(id.clone().into(), "CString conversion".to_string())
		})?;
		Ok(Shader(
			NonNull::new(unsafe { stereokit_sys::shader_find(c_str.as_ptr()) }).ok_or(
				StereoKitError::ShaderFind(id.clone().into(), "shader_find failed".to_string()),
			)?,
		))
	}

	/// Loads a shader from a precompiled StereoKit Shader (.sks) file! HLSL files can be compiled using the skshaderc tool included in the NuGet package. This should be taken care of by MsBuild automatically, but you may need to ensure your HLSL file is a item type in the .csproj for this to work. You can also compile with the command line app manually if you’re compiling/distributing a shader some other way!
	fn shader_create_file(&self, file: impl AsRef<Path>) -> SkResult<Shader> {
		let path = file.as_ref();
		let path_buf = path.to_path_buf();
		let c_str = CString::new(path_buf.clone().to_str().ok_or(StereoKitError::ShaderFile(
			path_buf.clone(),
			"CString conversion".to_string(),
		))?)
		.map_err(|_| {
			StereoKitError::ShaderFile(path_buf.clone(), "CString conversion".to_string())
		})?;
		Ok(Shader(
			NonNull::new(unsafe { stereokit_sys::shader_create_file(c_str.as_ptr()) }).ok_or(
				StereoKitError::ShaderFile(
					path_buf.clone(),
					"shader_create_file failed".to_string(),
				),
			)?,
		))
	}

	/// 	Creates a shader asset from a precompiled StereoKit Shader file stored as bytes!
	fn shader_create_mem(&self, data: &[u8]) -> SkResult<Shader> {
		Ok(Shader(
			NonNull::new(unsafe {
				stereokit_sys::shader_create_mem(data.as_ptr() as *mut c_void, data.len())
			})
			.ok_or(StereoKitError::ShaderMem)?,
		))
	}

	fn shader_set_id<S: Into<String> + Clone>(&self, shader: impl AsRef<Shader>, id: S) {
		let c_str = CString::new(id.into()).unwrap();
		unsafe { stereokit_sys::shader_set_id(shader.as_ref().0.as_ptr(), c_str.as_ptr()) }
	}

	fn shader_get_id(&self, shader: impl AsRef<Shader>) -> &str {
		unsafe { CStr::from_ptr(stereokit_sys::shader_get_id(shader.as_ref().0.as_ptr())) }
			.to_str()
			.unwrap()
	}

	/// The name of the shader, provided in the shader file itself. Not the filename or id.
	fn shader_get_name(&self, shader: impl AsRef<Shader>) -> &str {
		unsafe { CStr::from_ptr(stereokit_sys::shader_get_name(shader.as_ref().0.as_ptr())) }
			.to_str()
			.unwrap()
	}

	unsafe fn shader_addref(&self, shader: impl AsRef<Shader>) {
		unsafe { stereokit_sys::shader_addref(shader.as_ref().0.as_ptr()) }
	}

	fn shader_release(&self, _shader: Shader) {}

	/// Looks for a Material asset that’s already loaded, matching the given id!
	fn material_find<S: Into<String> + Clone>(&self, id: S) -> SkResult<Material> {
		let c_str = CString::new(id.clone().into()).map_err(|_| {
			StereoKitError::MaterialFind(id.clone().into(), "CString conversion".to_string())
		})?;
		Ok(Material(
			NonNull::new(unsafe { stereokit_sys::material_find(c_str.as_ptr()) }).ok_or(
				StereoKitError::MaterialFind(id.clone().into(), "material_find failed".to_string()),
			)?,
		))
	}

	/// Creates a material from a shader, and uses the shader’s default settings. Uses an auto-generated id.
	fn material_create(&self, shader: impl AsRef<Shader>) -> Material {
		Material(
			NonNull::new(unsafe { stereokit_sys::material_create(shader.as_ref().0.as_ptr()) })
				.unwrap(),
		)
	}

	/// Creates a new Material asset with the same shader and properties! Draw calls with the new Material will not batch together with this one.
	fn material_copy<M: AsRef<Material>>(&self, material: M) -> Material {
		Material(
			NonNull::new(unsafe { stereokit_sys::material_copy(material.as_ref().0.as_ptr()) })
				.unwrap(),
		)
	}

	fn material_copy_id<S: Into<String> + Clone>(&self, id: S) -> Material {
		let c_str = CString::new(id.into()).unwrap();
		Material(NonNull::new(unsafe { stereokit_sys::material_copy_id(c_str.as_ptr()) }).unwrap())
	}

	fn material_set_id<M: AsRef<Material>, S: Into<String> + Clone>(&self, material: M, id: S) {
		let c_str = CString::new(id.into()).unwrap();
		unsafe { stereokit_sys::material_set_id(material.as_ref().0.as_ptr(), c_str.as_ptr()) }
	}

	fn material_get_id<M: AsRef<Material>>(&self, material: M) -> &str {
		unsafe { CStr::from_ptr(stereokit_sys::material_get_id(material.as_ref().0.as_ptr())) }
			.to_str()
			.unwrap()
	}

	unsafe fn material_addref<M: AsRef<Material>>(&self, material: M) {
		unsafe { stereokit_sys::material_addref(material.as_ref().0.as_ptr()) }
	}

	fn material_release(&self, _material: Material) {}

	/// What type of transparency does this Material use? Default is None. Transparency has an impact on performance, and draw order. Check the Transparency enum for details.
	fn material_set_transparency<M: AsRef<Material>>(&self, material: M, mode: Transparency) {
		unsafe {
			stereokit_sys::material_set_transparency(
				material.as_ref().0.as_ptr(),
				mode as transparency_,
			);
		}
	}

	/// How should this material cull faces?
	fn material_set_cull<M: AsRef<Material>>(&self, material: M, mode: CullMode) {
		unsafe { stereokit_sys::material_set_cull(material.as_ref().0.as_ptr(), mode as cull_) }
	}

	/// Should this material draw only the edges/wires of the mesh? This can be useful for debugging, and even some kinds of visualization work. Note that this may not work on some mobile OpenGL systems like Quest.
	fn material_set_wireframe<M: AsRef<Material>>(&self, material: M, wireframe: bool) {
		unsafe {
			stereokit_sys::material_set_wireframe(
				material.as_ref().0.as_ptr(),
				wireframe as bool32_t,
			)
		}
	}

	/// How does this material interact with the ZBuffer? Generally DepthTest.Less would be normal behavior: don’t draw objects that are occluded. But this can also be used to achieve some interesting effects, like you could use DepthTest.Greater to draw a glow that indicates an object is behind something.
	fn material_set_depth_test<M: AsRef<Material>>(&self, material: M, depth_test_mode: DepthTest) {
		unsafe {
			stereokit_sys::material_set_depth_test(
				material.as_ref().0.as_ptr(),
				depth_test_mode as depth_test_,
			)
		}
	}

	/// Should this material write to the ZBuffer? For opaque objects, this generally should be true. But transparent objects writing to the ZBuffer can be problematic and cause draw order issues. Note that turning this off can mean that this material won’t get properly accounted for when the MR system is performing late stage reprojection. Not writing to the buffer can also be faster! :)
	fn material_set_depth_write<M: AsRef<Material>>(&self, material: M, write_enabled: bool) {
		unsafe {
			stereokit_sys::material_set_depth_write(
				material.as_ref().0.as_ptr(),
				write_enabled as bool32_t,
			)
		}
	}

	/// This property will force this material to draw earlier or later in the draw queue. Positive values make it draw later, negative makes it earlier. This can be helpful for tweaking performance! If you know an object is always going to be close to the user and likely to obscure lots of objects (like hands), drawing it earlier can mean objects behind it get discarded much faster! Similarly, objects that are far away (skybox!) can be pushed towards the back of the queue, so they’re more likely to be discarded early.
	fn material_set_queue_offset<M: AsRef<Material>>(&self, material: M, offset: i32) {
		unsafe { stereokit_sys::material_set_queue_offset(material.as_ref().0.as_ptr(), offset) }
	}

	fn material_set_chain<M: AsRef<Material>>(&self, material: M, chain_material: M) {
		unsafe {
			stereokit_sys::material_set_chain(
				material.as_ref().0.as_ptr(),
				chain_material.as_ref().0.as_ptr(),
			)
		}
	}

	/// What type of transparency does this Material use? Default is None. Transparency has an impact on performance, and draw order. Check the Transparency enum for details.
	fn material_get_transparency<M: AsRef<Material>>(&self, material: M) -> Transparency {
		unsafe {
			std::mem::transmute(stereokit_sys::material_get_transparency(
				material.as_ref().0.as_ptr(),
			))
		}
	}

	/// How should this material cull faces?
	fn material_get_cull<M: AsRef<Material>>(&self, material: M) -> CullMode {
		unsafe {
			std::mem::transmute(stereokit_sys::material_get_cull(
				material.as_ref().0.as_ptr(),
			))
		}
	}

	/// Should this material draw only the edges/wires of the mesh? This can be useful for debugging, and even some kinds of visualization work. Note that this may not work on some mobile OpenGL systems like Quest.
	fn material_get_wireframe<M: AsRef<Material>>(&self, material: M) -> bool {
		unsafe { stereokit_sys::material_get_wireframe(material.as_ref().0.as_ptr()) != 0 }
	}

	/// How does this material interact with the ZBuffer? Generally DepthTest.Less would be normal behavior: don’t draw objects that are occluded. But this can also be used to achieve some interesting effects, like you could use DepthTest.Greater to draw a glow that indicates an object is behind something.
	fn material_get_depth_test<M: AsRef<Material>>(&self, material: M) -> DepthTest {
		unsafe {
			std::mem::transmute(stereokit_sys::material_get_depth_test(
				material.as_ref().0.as_ptr(),
			))
		}
	}

	/// Should this material write to the ZBuffer? For opaque objects, this generally should be true. But transparent objects writing to the ZBuffer can be problematic and cause draw order issues. Note that turning this off can mean that this material won’t get properly accounted for when the MR system is performing late stage reprojection. Not writing to the buffer can also be faster! :)
	fn material_get_depth_write<M: AsRef<Material>>(&self, material: M) -> bool {
		unsafe { stereokit_sys::material_get_depth_write(material.as_ref().0.as_ptr()) != 0 }
	}

	/// This property will force this material to draw earlier or later in the draw queue. Positive values make it draw later, negative makes it earlier. This can be helpful for tweaking performance! If you know an object is always going to be close to the user and likely to obscure lots of objects (like hands), drawing it earlier can mean objects behind it get discarded much faster! Similarly, objects that are far away (skybox!) can be pushed towards the back of the queue, so they’re more likely to be discarded early.
	fn material_get_queue_offset<M: AsRef<Material>>(&self, material: M) -> i32 {
		unsafe { stereokit_sys::material_get_queue_offset(material.as_ref().0.as_ptr()) }
	}

	fn material_get_chain<M: AsRef<Material>>(&self, material: M) -> Option<Material> {
		Some(Material(NonNull::new(unsafe {
			stereokit_sys::material_get_chain(material.as_ref().0.as_ptr())
		})?))
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_float<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: f32,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_float(material.as_ref().0.as_ptr(), c_str.as_ptr(), value)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_vector2<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: impl Into<Vec2>,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_vector2(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value.into().into(),
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_vector3<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: impl Into<Vec3>,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_vector3(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value.into().into(),
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_color<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: Color128,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_color(material.as_ref().0.as_ptr(), c_str.as_ptr(), value)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_vector4<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: impl Into<Vec4>,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_vector4(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value.into().into(),
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_vector<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: impl Into<Vec4>,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_vector(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value.into().into(),
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_int<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: i32,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_int(material.as_ref().0.as_ptr(), c_str.as_ptr(), value)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_int2<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value1: i32,
		value2: i32,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_int2(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value1,
				value2,
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_int3<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value1: i32,
		value2: i32,
		value3: i32,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_int3(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value1,
				value2,
				value3,
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_int4<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value1: i32,
		value2: i32,
		value3: i32,
		value4: i32,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_int4(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value1,
				value2,
				value3,
				value4,
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_bool<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: bool,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_bool(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value as bool32_t,
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_uint<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: u32,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_uint(material.as_ref().0.as_ptr(), c_str.as_ptr(), value)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_uint2<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value1: u32,
		value2: u32,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_uint2(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value1,
				value2,
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_uint3<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value1: u32,
		value2: u32,
		value3: u32,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_uint3(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value1,
				value2,
				value3,
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_uint4<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value1: u32,
		value2: u32,
		value3: u32,
		value4: u32,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_uint4(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value1,
				value2,
				value3,
				value4,
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_matrix<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: impl Into<Mat4>,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_matrix(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value.into().into(),
			)
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_texture<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		value: impl AsRef<Tex>,
	) -> bool {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_texture(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				value.as_ref().0.as_ptr(),
			) != 0
		}
	}

	/// Sets a shader parameter with the given name to the provided value. If no parameter is found, nothing happens, and the value is not set!
	fn material_set_texture_id<M: AsRef<Material>, T: AsRef<Tex>>(
		&self,
		material: M,
		id: u64,
		tex: T,
	) -> bool {
		unsafe {
			stereokit_sys::material_set_texture_id(
				material.as_ref().0.as_ptr(),
				id,
				tex.as_ref().0.as_ptr(),
			) != 0
		}
	}

	fn material_get_float<M: AsRef<Material>, S: AsRef<str>>(&self, material: M, name: S) -> f32 {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe { stereokit_sys::material_get_float(material.as_ref().0.as_ptr(), c_str.as_ptr()) }
	}

	fn material_get_vector2<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
	) -> Vec2 {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe { stereokit_sys::material_get_vector2(material.as_ref().0.as_ptr(), c_str.as_ptr()) }
			.into()
	}

	fn material_get_vector3<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
	) -> Vec3 {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe { stereokit_sys::material_get_vector3(material.as_ref().0.as_ptr(), c_str.as_ptr()) }
			.into()
	}

	fn material_get_vector4<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
	) -> Vec4 {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe { stereokit_sys::material_get_vector4(material.as_ref().0.as_ptr(), c_str.as_ptr()) }
			.into()
	}

	fn material_get_int<M: AsRef<Material>, S: AsRef<str>>(&self, material: M, name: S) -> i32 {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe { stereokit_sys::material_get_int(material.as_ref().0.as_ptr(), c_str.as_ptr()) }
	}

	fn material_get_bool<M: AsRef<Material>, S: AsRef<str>>(&self, material: M, name: S) -> bool {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_get_bool(material.as_ref().0.as_ptr(), c_str.as_ptr()) != 0
		}
	}

	fn material_get_uint<M: AsRef<Material>, S: AsRef<str>>(&self, material: M, name: S) -> u32 {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe { stereokit_sys::material_get_uint(material.as_ref().0.as_ptr(), c_str.as_ptr()) }
	}

	fn material_get_matrix<M: AsRef<Material>, S: AsRef<str>>(&self, material: M, name: S) -> Mat4 {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe { stereokit_sys::material_get_matrix(material.as_ref().0.as_ptr(), c_str.as_ptr()) }
			.into()
	}

	fn material_get_texture<M: AsRef<Material>, S: AsRef<str>>(&self, material: M, name: S) -> Tex {
		let c_str = CString::new(name.as_ref()).unwrap();
		Tex(NonNull::new(unsafe {
			stereokit_sys::material_get_texture(material.as_ref().0.as_ptr(), c_str.as_ptr())
		})
		.unwrap())
	}

	fn material_has_param<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		type_: MaterialParameter,
	) -> bool {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_has_param(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				std::mem::transmute(type_),
			) != 0
		}
	}

	unsafe fn material_set_param<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		type_: MaterialParameter,
		value: *const std::os::raw::c_void,
	) {
		let c_str = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::material_set_param(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				std::mem::transmute(type_),
				value,
			)
		}
	}

	unsafe fn material_set_param_id<M: AsRef<Material>>(
		&self,
		material: M,
		id: u64,
		type_: MaterialParameter,
		value: *const std::os::raw::c_void,
	) {
		unsafe {
			stereokit_sys::material_set_param_id(
				material.as_ref().0.as_ptr(),
				id,
				std::mem::transmute(type_),
				value,
			)
		}
	}

	unsafe fn material_get_param<M: AsRef<Material>, S: AsRef<str>>(
		&self,
		material: M,
		name: S,
		type_: MaterialParameter,
	) -> Option<*mut std::os::raw::c_void> {
		let c_str = CString::new(name.as_ref()).unwrap();
		let value = null_mut();
		match unsafe {
			stereokit_sys::material_get_param(
				material.as_ref().0.as_ptr(),
				c_str.as_ptr(),
				std::mem::transmute(type_),
				value,
			) != 0
		} {
			true => Some(value),
			false => None,
		}
	}

	unsafe fn material_get_param_id<M: AsRef<Material>>(
		&self,
		material: M,
		id: u64,
		type_: MaterialParameter,
	) -> Option<*mut std::os::raw::c_void> {
		let value = null_mut();
		match unsafe {
			stereokit_sys::material_get_param_id(
				material.as_ref().0.as_ptr(),
				id,
				std::mem::transmute(type_),
				value,
			) != 0
		} {
			true => Some(value),
			false => None,
		}
	}

	//TODO: material_get_param_info need to figure out how to do this one

	fn material_get_param_count<M: AsRef<Material>>(&self, material: M) -> i32 {
		unsafe { stereokit_sys::material_get_param_count(material.as_ref().0.as_ptr()) }
	}

	fn material_set_shader<M: AsRef<Material>>(&self, material: M, shader: impl AsRef<Shader>) {
		unsafe {
			stereokit_sys::material_set_shader(
				material.as_ref().0.as_ptr(),
				shader.as_ref().0.as_ptr(),
			)
		}
	}

	fn material_get_shader<M: AsRef<Material>>(&self, material: M) -> Shader {
		Shader(
			NonNull::new(unsafe {
				stereokit_sys::material_get_shader(material.as_ref().0.as_ptr())
			})
			.unwrap(),
		)
	}

	fn material_buffer_create(&self, register_slot: i32, size: i32) -> MaterialBuffer {
		MaterialBuffer(
			NonNull::new(unsafe { stereokit_sys::material_buffer_create(register_slot, size) })
				.unwrap(),
		)
	}

	unsafe fn material_buffer_set_data<MB: AsRef<MaterialBuffer>>(
		&self,
		material_buffer: MB,
		buffer_data: *const std::os::raw::c_void,
	) {
		stereokit_sys::material_buffer_set_data(material_buffer.as_ref().0.as_ptr(), buffer_data)
	}

	fn material_buffer_release(&self, _material_buffer: MaterialBuffer) {}

	///this is unsafe because there is no way to release text_styles, so by calling the function repeatedly memory leaks occur
	/// Create a text style for use with other text functions! A text style is a font plus size/color/material parameters, and are used to keep text looking more consistent through the application by encouraging devs to re-use styles throughout the project. This overload will create a unique Material for this style based on Default.ShaderFont.
	unsafe fn text_make_style<F: AsRef<Font>>(
		&self,
		font: F,
		character_height: f32,
		color_gamma: Color128,
	) -> TextStyle {
		TextStyle(stereokit_sys::text_make_style(
			font.as_ref().0.as_ptr(),
			character_height,
			color_gamma,
		))
	}

	unsafe fn text_make_style_shader<F: AsRef<Font>>(
		&self,
		font: F,
		character_height: f32,
		shader: impl AsRef<Shader>,
		color_gamma: Color128,
	) -> TextStyle {
		TextStyle(stereokit_sys::text_make_style_shader(
			font.as_ref().0.as_ptr(),
			character_height,
			shader.as_ref().0.as_ptr(),
			color_gamma,
		))
	}

	unsafe fn text_make_style_mat<F: AsRef<Font>, M: AsRef<Material>>(
		&self,
		font: F,
		character_height: f32,
		material: M,
		color_gamma: Color128,
	) -> TextStyle {
		TextStyle(stereokit_sys::text_make_style_mat(
			font.as_ref().0.as_ptr(),
			character_height,
			material.as_ref().0.as_ptr(),
			color_gamma,
		))
	}

	/// Renders text at the given location! Must be called every frame you want this text to be visible.
	fn text_add_at(
		&self,
		text_utf8: impl AsRef<str>,
		transform: impl Into<Mat4>,
		style: TextStyle,
		position: TextAlign,
		align: TextAlign,
		offset: impl Into<Vec3>,
		vertex_tint_linear: Color128,
	) {
		let c_str = CString::new(text_utf8.as_ref()).unwrap();
		let offset = offset.into();
		let transform = transform.into().into();
		unsafe {
			stereokit_sys::text_add_at(
				c_str.as_ptr(),
				&transform,
				style.0,
				position.into(),
				align.into(),
				offset.x,
				offset.y,
				offset.z,
				vertex_tint_linear,
			)
		}
	}

	//TODO: text_add_at_16, have to add utf16 support

	fn text_add_in(
		&self,
		text_utf8: impl AsRef<str>,
		transform: impl Into<Mat4>,
		size: impl Into<Vec2>,
		fit: TextFit,
		style: TextStyle,
		position: TextAlign,
		align: TextAlign,
		offset: impl Into<Vec3>,
		vertex_tint_linear: Color128,
	) -> f32 {
		let text_utf8 = CString::new(text_utf8.as_ref()).unwrap();
		let transform = transform.into().into();
		let size = size.into().into();
		let offset = offset.into();
		unsafe {
			stereokit_sys::text_add_in(
				text_utf8.as_ptr(),
				&transform,
				size,
				fit.into(),
				style.0,
				position.into(),
				align.into(),
				offset.x,
				offset.y,
				offset.z,
				vertex_tint_linear,
			)
		}
	}

	/// Sometimes you just need to know how much room some text takes up! This finds the size of the text in meters when using the indicated style!
	fn text_size<S: AsRef<str>>(&self, text_utf8: S, style: TextStyle) -> Vec2 {
		let text_utf8 = CString::new(text_utf8.as_ref()).unwrap();
		unsafe { stereokit_sys::text_size(text_utf8.as_ptr(), style.0) }.into()
	}

	//TODO: text_size_16

	//TODO: text_char_at

	//TODO: text_char_at_16

	fn text_style_get_material(&self, style: TextStyle) -> Material {
		Material(NonNull::new(unsafe { stereokit_sys::text_style_get_material(style.0) }).unwrap())
	}

	fn text_style_get_char_height(&self, style: TextStyle) -> f32 {
		unsafe { stereokit_sys::text_style_get_char_height(style.0) }
	}

	/// Looks for a Model asset that’s already loaded, matching the given id!
	fn model_find<S: Into<String> + Clone>(&self, id: S) -> SkResult<Model> {
		let str = std::ffi::CString::new(id.clone().into())
			.map_err(|_| StereoKitError::ModelFile(id.clone().into()))?;
		Ok(Model::from(
			NonNull::new(unsafe { stereokit_sys::model_find(str.as_ptr()) })
				.ok_or(StereoKitError::ModelFile(id.clone().into()))?,
		))
	}
	/// Creates a shallow copy of a Model asset! Meshes and Materials
	/// referenced by this Model will be referenced, not copied.
	fn model_copy<M: AsRef<Model>>(&self, model: M) -> Model {
		Model::from(
			NonNull::new(unsafe { stereokit_sys::model_copy(model.as_ref().0.as_ptr()) }).unwrap(),
		)
	}

	fn model_create(&self) -> Model {
		Model::from(NonNull::new(unsafe { stereokit_sys::model_create() }).unwrap())
	}

	/// Creates a single mesh subset Model using the indicated Mesh and Material!
	/// An id will be automatically generated for this asset.
	fn model_create_mesh<Me: AsRef<Mesh>, Ma: AsRef<Material>>(
		&self,
		mesh: Me,
		material: Ma,
	) -> Model {
		Model::from(
			NonNull::new(unsafe {
				stereokit_sys::model_create_mesh(
					mesh.as_ref().0.as_ptr(),
					material.as_ref().0.as_ptr(),
				)
			})
			.unwrap(),
		)
	}
	/// Loads a list of mesh and material subsets from a .obj, .stl, .ply (ASCII),
	/// .gltf, or .glb file stored in memory. Note that this function won’t work
	/// well on files that reference other files, such as .gltf files with
	/// references in them.
	fn model_create_mem<S: Into<String> + Clone>(
		&self,
		file_name: S,
		memory: &[u8],
		shader: Option<impl AsRef<Shader>>,
	) -> SkResult<Model> {
		let c_file_name = std::ffi::CString::new(file_name.clone().into()).map_err(|_| {
			StereoKitError::ModelFromMem(
				file_name.clone().into(),
				String::from("file name is not a valid CString"),
			)
		})?;
		Ok(Model::from(
			NonNull::new(unsafe {
				stereokit_sys::model_create_mem(
					c_file_name.as_ptr(),
					memory.as_ptr() as *mut c_void,
					memory.len(),
					shader.map(|shader| shader.as_ref().0.as_ptr()).unwrap_or(null_mut()),
				)
			})
			.ok_or(StereoKitError::ModelFromMem(
				file_name.clone().into(),
				String::from("model_create_mem failed"),
			))?,
		))
	}

	/// Loads a list of mesh and material subsets from a .obj, .stl, .ply (ASCII), .gltf, or .glb file.
	fn model_create_file(
		&self,
		filename: impl AsRef<Path>,
		shader: Option<impl AsRef<Shader>>,
	) -> SkResult<Model> {
		let path = filename.as_ref();
		let path_buf = path.to_path_buf();
		let c_str = CString::new(path.to_str().ok_or(StereoKitError::TexFile(
			path_buf.clone(),
			"CString conversion".to_string(),
		))?)
		.map_err(|_| StereoKitError::TexFile(path_buf.clone(), "CString Conversion".to_string()))?;
		Ok(Model::from(
			NonNull::new(unsafe {
				stereokit_sys::model_create_file(
					c_str.as_ptr(),
					shader.map(|s| s.as_ref().0.as_ptr()).unwrap_or(null_mut()),
				)
			})
			.ok_or(StereoKitError::ModelFromFile(
				path_buf.clone(),
				"model_create_file failed".to_string(),
			))?,
		))
	}

	fn model_set_id<M: AsRef<Model>, S: AsRef<str>>(&self, model: M, id: S) {
		let id = CString::new(id.as_ref()).unwrap();
		unsafe { stereokit_sys::model_set_id(model.as_ref().0.as_ptr(), id.as_ptr()) }
	}

	fn model_get_id<M: AsRef<Model>>(&self, model: M) -> &str {
		unsafe { CStr::from_ptr(stereokit_sys::model_get_id(model.as_ref().0.as_ptr())) }
			.to_str()
			.unwrap()
	}

	unsafe fn model_addref<M: AsRef<Model>>(&self, model: M) {
		stereokit_sys::model_addref(model.as_ref().0.as_ptr())
	}

	fn model_release(&self, _model: Model) {}

	/// Examines the visuals as they currently are, and rebuilds the bounds based on that! This is normally done automatically, but if you modify a Mesh that this Model is using, the Model can’t see it, and you should call this manually.
	fn model_recalculate_bounds<M: AsRef<Model>>(&self, model: M) {
		unsafe { stereokit_sys::model_recalculate_bounds(model.as_ref().0.as_ptr()) }
	}

	/// Examines the visuals as they currently are, and rebuilds the bounds based on all the vertices in the model! This leads (in general) to a tighter bound than the default bound based on bounding boxes. However, computing the exact bound can take much longer!
	fn model_recalculate_bounds_exact<M: AsRef<Model>>(&self, model: M) {
		unsafe { stereokit_sys::model_recalculate_bounds_exact(model.as_ref().0.as_ptr()) }
	}

	fn model_set_bounds<M: AsRef<Model>>(&self, model: M, bounds: impl AsRef<Bounds>) {
		let bounds = bounds.as_ref();
		unsafe {
			stereokit_sys::model_set_bounds(
				model.as_ref().0.as_ptr(),
				std::mem::transmute(bounds as *const Bounds),
			)
		}
	}

	fn model_get_bounds<M: AsRef<Model>>(&self, model: M) -> Bounds {
		unsafe {
			std::mem::transmute(stereokit_sys::model_get_bounds(model.as_ref().0.as_ptr()))
		}
	}

	/// Calling Draw will automatically step the Model’s animation, but if you don’t draw the Model, or need access to the animated nodes before drawing, then you can step the animation early manually via this method. Animation will only ever be stepped once per frame, so it’s okay to call this multiple times, or in addition to Draw.
	fn model_step_anim<M: AsRef<Model>>(&self, model: M) {
		unsafe { stereokit_sys::model_step_anim(model.as_ref().0.as_ptr()) }
	}

	/// Searches for an animation with the given name, and if it’s found, sets it up as the active animation and begins playing it with the animation mode.
	fn model_play_anim<M: AsRef<Model>>(
		&self,
		model: M,
		animation_name: impl AsRef<str>,
		mode: AnimMode,
	) -> bool {
		let animation_name = CString::new(animation_name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::model_play_anim(
				model.as_ref().0.as_ptr(),
				animation_name.as_ptr(),
				mode as anim_mode_,
			) != 0
		}
	}

	fn model_play_anim_idx<M: AsRef<Model>>(&self, model: M, index: i32, mode: AnimMode) {
		unsafe {
			stereokit_sys::model_play_anim_idx(model.as_ref().0.as_ptr(), index, mode as anim_mode_)
		}
	}

	fn model_set_anim_time<M: AsRef<Model>>(&self, model: M, time: f32) {
		unsafe { stereokit_sys::model_set_anim_time(model.as_ref().0.as_ptr(), time) }
	}

	fn model_set_anim_completion<M: AsRef<Model>>(&self, model: M, percent: f32) {
		unsafe { stereokit_sys::model_set_anim_completion(model.as_ref().0.as_ptr(), percent) }
	}

	/// Searches the list of animations for the first one matching the given name.
	fn model_anim_find<M: AsRef<Model>, S: AsRef<str>>(&self, model: M, animation_name: S) -> i32 {
		let animation_name = CString::new(animation_name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::model_anim_find(model.as_ref().0.as_ptr(), animation_name.as_ptr())
		}
	}

	fn model_anim_count<M: AsRef<Model>>(&self, model: M) -> i32 {
		unsafe { stereokit_sys::model_anim_count(model.as_ref().0.as_ptr()) }
	}

	fn model_anim_active<M: AsRef<Model>>(&self, model: M) -> i32 {
		unsafe { stereokit_sys::model_anim_active(model.as_ref().0.as_ptr()) }
	}

	fn model_anim_active_mode<M: AsRef<Model>>(&self, model: M) -> AnimMode {
		unsafe {
			std::mem::transmute(stereokit_sys::model_anim_active_mode(
				model.as_ref().0.as_ptr(),
			))
		}
	}

	fn model_anim_active_time<M: AsRef<Model>>(&self, model: M) -> f32 {
		unsafe { stereokit_sys::model_anim_active_time(model.as_ref().0.as_ptr()) }
	}

	fn model_anim_active_completion<M: AsRef<Model>>(&self, model: M) -> f32 {
		unsafe { stereokit_sys::model_anim_active_completion(model.as_ref().0.as_ptr()) }
	}

	fn model_anim_get_name<M: AsRef<Model>>(&self, model: M, index: i32) -> Option<&str> {
		unsafe {
			CStr::from_ptr(stereokit_sys::model_anim_get_name(
				model.as_ref().0.as_ptr(),
				index,
			))
		}
		.to_str()
		.map(|a| Some(a))
		.unwrap_or(None)
	}

	fn model_anim_get_duration<M: AsRef<Model>>(&self, model: M, index: i32) -> f32 {
		unsafe { stereokit_sys::model_anim_get_duration(model.as_ref().0.as_ptr(), index) }
	}

	/// Returns the name of the specific subset! This will be the node name of your model asset. If no node name is available, SteroKit will generate a name in the format of “subsetX”, where X would be the subset index. Note that names are not guaranteed to be unique (users may assign the same name to multiple nodes). Some nodes may also produce multiple subsets with the same name, such as when a node contains a Mesh with multiple Materials, each Mesh/Material combination will receive a subset with the same name.
	fn model_get_name<M: AsRef<Model>>(&self, model: M, subset: i32) -> &str {
		unsafe {
			CStr::from_ptr(stereokit_sys::model_get_name(
				model.as_ref().0.as_ptr(),
				subset,
			))
		}
		.to_str()
		.unwrap()
	}

	/// Gets a link to the Material asset used by the model subset! Note that this is not necessarily a unique material, and could be shared in a number of other places. Consider copying and replacing it if you intend to modify it!
	fn model_get_material<M: AsRef<Model>>(&self, model: M, subset: i32) -> Option<Material> {
		Some(Material(NonNull::new(unsafe {
			stereokit_sys::model_get_material(model.as_ref().0.as_ptr(), subset)
		})?))
	}

	/// Gets a link to the Mesh asset used by the model subset! Note that this is not necessarily a unique mesh, and could be shared in a number of other places. Consider copying and replacing it if you intend to modify it!
	fn model_get_mesh<M: AsRef<Model>>(&self, model: M, subset: i32) -> Option<Mesh> {
		Some(Mesh(NonNull::new(unsafe {
			stereokit_sys::model_get_mesh(model.as_ref().0.as_ptr(), subset)
		})?))
	}

	///	Gets the transform matrix used by the model subset!
	fn model_get_transform<M: AsRef<Model>>(&self, model: M, subset: i32) -> Mat4 {
		unsafe { stereokit_sys::model_get_transform(model.as_ref().0.as_ptr(), subset) }.into()
	}

	/// Changes the Material for the subset to a new one!
	fn model_set_material<M: AsRef<Model>>(
		&self,
		model: M,
		subset: i32,
		material: impl AsRef<Material>,
	) {
		unsafe {
			stereokit_sys::model_set_material(
				model.as_ref().0.as_ptr(),
				subset,
				material.as_ref().0.as_ptr(),
			)
		}
	}

	/// Changes the mesh for the subset to a new one!
	fn model_set_mesh<M: AsRef<Model>>(&self, model: M, subset: i32, mesh: impl AsRef<Mesh>) {
		unsafe {
			stereokit_sys::model_set_mesh(
				model.as_ref().0.as_ptr(),
				subset,
				mesh.as_ref().0.as_ptr(),
			)
		}
	}

	/// Changes the transform for the subset to a new one! This is in Model space, so it’s relative to the origin of the model.
	fn model_set_transform<M: AsRef<Model>>(
		&self,
		model: M,
		subset: i32,
		transform: impl Into<Mat4>,
	) {
		let transform = transform.into().into();
		unsafe { stereokit_sys::model_set_transform(model.as_ref().0.as_ptr(), subset, &transform) }
	}

	/// Removes and dereferences a subset from the model.
	fn model_remove_subset<M: AsRef<Model>>(&self, model: M, subset: i32) {
		unsafe { stereokit_sys::model_remove_subset(model.as_ref().0.as_ptr(), subset) }
	}

	/// Adds a new subset to the Model, and recalculates the bounds. A default subset name of “subsetX” will be used, where X is the subset’s index.
	fn model_add_named_subset<M: AsRef<Model>, S: AsRef<str>>(
		&self,
		model: M,
		name: S,
		mesh: impl AsRef<Mesh>,
		material: impl AsRef<Material>,
		transform: impl Into<Mat4>,
	) -> i32 {
		let name = CString::new(name.as_ref()).unwrap();
		let transform = transform.into().into();
		unsafe {
			stereokit_sys::model_add_named_subset(
				model.as_ref().0.as_ptr(),
				name.as_ptr(),
				mesh.as_ref().0.as_ptr(),
				material.as_ref().0.as_ptr(),
				&transform,
			)
		}
	}

	/// Adds a new subset to the Model, and recalculates the bounds. A default subset name of “subsetX” will be used, where X is the subset’s index.
	fn model_add_subset<M: AsRef<Model>>(
		&self,
		model: M,
		mesh: impl AsRef<Mesh>,
		material: impl AsRef<Material>,
		transform: impl Into<Mat4>,
	) -> i32 {
		let transform = transform.into().into();
		unsafe {
			stereokit_sys::model_add_subset(
				model.as_ref().0.as_ptr(),
				mesh.as_ref().0.as_ptr(),
				material.as_ref().0.as_ptr(),
				&transform,
			)
		}
	}

	fn model_subset_count<M: AsRef<Model>>(&self, model: M) -> i32 {
		unsafe { stereokit_sys::model_subset_count(model.as_ref().0.as_ptr()) }
	}

	/// This adds a root node to the Model’s node hierarchy! If There is already an initial root node, this node will still be a root node, but will be a Sibling of the Model’s RootNode. If this is the first root node added, you’ll be able to access it via RootNode.
	fn model_node_add<M: AsRef<Model>, S: AsRef<str>>(
		&self,
		model: M,
		name: S,
		model_transform: impl Into<Mat4>,
		mesh: impl AsRef<Mesh>,
		material: impl AsRef<Material>,
	) -> ModelNodeId {
		let name = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::model_node_add(
				model.as_ref().0.as_ptr(),
				name.as_ptr(),
				model_transform.into().into(),
				mesh.as_ref().0.as_ptr(),
				material.as_ref().0.as_ptr(),
				0,
			)
		}
	}

	/// Adds a Child node below this node, at the end of the child chain!
	fn model_node_add_child<M: AsRef<Model>, S: AsRef<str>>(
		&self,
		model: M,
		parent: ModelNodeId,
		name: S,
		local_transform: impl Into<Mat4>,
		mesh: impl AsRef<Mesh>,
		material: impl AsRef<Material>,
	) -> ModelNodeId {
		let name = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::model_node_add_child(
				model.as_ref().0.as_ptr(),
				parent,
				name.as_ptr(),
				local_transform.into().into(),
				mesh.as_ref().0.as_ptr(),
				material.as_ref().0.as_ptr(),
				0,
			)
		}
	}

	fn model_node_find<M: AsRef<Model>, S: AsRef<str>>(
		&self,
		model: M,
		name: S,
	) -> Option<ModelNodeId> {
		let name = CString::new(name.as_ref()).unwrap();
		match unsafe { stereokit_sys::model_node_find(model.as_ref().0.as_ptr(), name.as_ptr()) } {
			-1 => None,
			otherwise => Some(otherwise),
		}
	}

	/// The next ModelNode in the hierarchy, at the same level as this one. To the “right” on a hierarchy tree. None if there are no more ModelNodes in the tree there.
	fn model_node_sibling<M: AsRef<Model>>(
		&self,
		model: M,
		node: ModelNodeId,
	) -> Option<ModelNodeId> {
		match unsafe { stereokit_sys::model_node_sibling(model.as_ref().0.as_ptr(), node) } {
			-1 => None,
			otherwise => Some(otherwise),
		}
	}

	/// The ModelNode above this one (“up”) in the hierarchy tree, or None if this is a root node.
	fn model_node_parent<M: AsRef<Model>>(
		&self,
		model: M,
		node: ModelNodeId,
	) -> Option<ModelNodeId> {
		match unsafe { stereokit_sys::model_node_parent(model.as_ref().0.as_ptr(), node) } {
			-1 => None,
			otherwise => Some(otherwise),
		}
	}

	/// The first child node “below” on the hierarchy tree, or None if there are none. To see all children, get the Child and then iterate through its Siblings.
	fn model_node_child<M: AsRef<Model>>(
		&self,
		model: M,
		node: ModelNodeId,
	) -> Option<ModelNodeId> {
		match unsafe { stereokit_sys::model_node_child(model.as_ref().0.as_ptr(), node) } {
			-1 => None,
			otherwise => Some(otherwise),
		}
	}

	fn model_node_count<M: AsRef<Model>>(&self, model: M) -> i32 {
		unsafe { stereokit_sys::model_node_count(model.as_ref().0.as_ptr()) }
	}

	fn model_node_index<M: AsRef<Model>>(&self, model: M, index: i32) -> Option<ModelNodeId> {
		match unsafe { stereokit_sys::model_node_index(model.as_ref().0.as_ptr(), index) } {
			-1 => None,
			otherwise => Some(otherwise),
		}
	}

	fn model_node_visual_count<M: AsRef<Model>>(&self, model: M) -> i32 {
		unsafe { stereokit_sys::model_node_visual_count(model.as_ref().0.as_ptr()) }
	}

	fn model_node_visual_index<M: AsRef<Model>>(&self, model: M, index: i32) -> Option<ModelNodeId> {
		match unsafe { stereokit_sys::model_node_visual_index(model.as_ref().0.as_ptr(), index) } {
			-1 => None,
			otherwise => Some(otherwise),
		}
	}

	fn model_node_iterate<M: AsRef<Model>>(
		&self,
		model: M,
		node: ModelNodeId,
	) -> Option<ModelNodeId> {
		match unsafe { stereokit_sys::model_node_iterate(model.as_ref().0.as_ptr(), node) } {
			-1 => None,
			otherwise => Some(otherwise),
		}
	}

	fn model_node_get_root<M: AsRef<Model>>(&self, model: M) -> ModelNodeId {
		unsafe { stereokit_sys::model_node_get_root(model.as_ref().0.as_ptr()) }
	}

	fn model_node_get_name<M: AsRef<Model>>(&self, model: M, node: ModelNodeId) -> Option<&str> {
		unsafe {
			CStr::from_ptr(stereokit_sys::model_node_get_name(
				model.as_ref().0.as_ptr(),
				node,
			))
			.to_str()
			.map(|s| Some(s))
			.unwrap_or(None)
		}
	}

	fn model_node_get_solid<M: AsRef<Model>>(&self, model: M, node: ModelNodeId) -> bool {
		unsafe { stereokit_sys::model_node_get_solid(model.as_ref().0.as_ptr(), node) != 0 }
	}

	fn model_node_get_visible<M: AsRef<Model>>(&self, model: M, node: ModelNodeId) -> bool {
		unsafe { stereokit_sys::model_node_get_visible(model.as_ref().0.as_ptr(), node) != 0 }
	}

	fn model_node_get_material<M: AsRef<Model>>(
		&self,
		model: M,
		node: ModelNodeId,
	) -> Option<Material> {
		Some(Material(NonNull::new(unsafe {
			stereokit_sys::model_node_get_material(model.as_ref().0.as_ptr(), node)
		})?))
	}

	fn model_node_get_mesh<M: AsRef<Model>>(&self, model: M, node: ModelNodeId) -> Option<Mesh> {
		Some(Mesh(NonNull::new(unsafe {
			stereokit_sys::model_node_get_mesh(model.as_ref().0.as_ptr(), node)
		})?))
	}

	fn model_node_get_transform_model<M: AsRef<Model>>(&self, model: M, node: ModelNodeId) -> Mat4 {
		unsafe { stereokit_sys::model_node_get_transform_model(model.as_ref().0.as_ptr(), node) }
			.into()
	}

	fn model_node_get_transform_local<M: AsRef<Model>>(&self, model: M, node: ModelNodeId) -> Mat4 {
		unsafe { stereokit_sys::model_node_get_transform_local(model.as_ref().0.as_ptr(), node) }
			.into()
	}

	fn model_node_set_name<M: AsRef<Model>, S: AsRef<str>>(
		&self,
		model: M,
		node: ModelNodeId,
		name: S,
	) {
		let name = CString::new(name.as_ref()).unwrap();
		unsafe {
			stereokit_sys::model_node_set_name(model.as_ref().0.as_ptr(), node, name.as_ptr())
		}
	}

	fn model_node_set_solid<M: AsRef<Model>>(&self, model: M, node: ModelNodeId, solid: bool) {
		unsafe {
			stereokit_sys::model_node_set_solid(
				model.as_ref().0.as_ptr(),
				node,
				solid as bool32_t,
			)
		}
	}

	fn model_node_set_visible<M: AsRef<Model>>(&self, model: M, node: ModelNodeId, visible: bool) {
		unsafe {
			stereokit_sys::model_node_set_visible(
				model.as_ref().0.as_ptr(),
				node,
				visible as bool32_t,
			)
		}
	}

	fn model_node_set_material<M: AsRef<Model>>(
		&self,
		model: M,
		node: ModelNodeId,
		material: impl AsRef<Material>,
	) {
		unsafe {
			stereokit_sys::model_node_set_material(
				model.as_ref().0.as_ptr(),
				node,
				material.as_ref().0.as_ptr(),
			)
		}
	}

	fn model_node_set_mesh<M: AsRef<Model>>(
		&self,
		model: M,
		node: ModelNodeId,
		mesh: impl AsRef<Mesh>,
	) {
		unsafe {
			stereokit_sys::model_node_set_mesh(
				model.as_ref().0.as_ptr(),
				node,
				mesh.as_ref().0.as_ptr(),
			)
		}
	}

	fn model_node_set_transform_model<M: AsRef<Model>>(
		&self,
		model: M,
		node: ModelNodeId,
		transform_model_space: impl Into<Mat4>,
	) {
		unsafe {
			stereokit_sys::model_node_set_transform_model(
				model.as_ref().0.as_ptr(),
				node,
				transform_model_space.into().into(),
			)
		}
	}

	fn model_node_set_transform_local<M: AsRef<Model>>(
		&self,
		model: M,
		node: ModelNodeId,
		transform_local_space: impl Into<Mat4>,
	) {
		unsafe {
			stereokit_sys::model_node_set_transform_local(
				model.as_ref().0.as_ptr(),
				node,
				transform_local_space.into().into(),
			)
		}
	}

	fn model_node_info_get<M: AsRef<Model>, S: AsRef<str>>(
		&self,
		model: M,
		node: ModelNodeId,
		info_key_utf8: S,
	) -> Option<&str> {
		let info_key_utf8_c = CString::new(info_key_utf8.as_ref()).unwrap();
		match NonNull::new (unsafe {
			stereokit_sys::model_node_info_get(
				model.as_ref().0.as_ptr(), 
				node,
				info_key_utf8_c.as_ptr(),
			) 
		})  {
		    Some(non_null) => return unsafe{CStr::from_ptr(non_null.as_ref()).to_str().ok()},
    		None => None,
		}

	}

	fn model_node_info_set<M: AsRef<Model>, S: AsRef<str>>(
		&self,
		model: M,
		node: ModelNodeId,
		info_key_utf8: S,
		info_value_utf8: S,
	) {
		let info_key_utf8 = CString::new(info_key_utf8.as_ref()).unwrap();
		let info_value_utf8 = CString::new(info_value_utf8.as_ref()).unwrap();
		unsafe {
			stereokit_sys::model_node_info_set(
				model.as_ref().0.as_ptr(),
				node,
				info_key_utf8.as_ptr(),
				info_value_utf8.as_ptr(),
			)
		}
	}

	fn model_node_info_remove<M: AsRef<Model>, S: AsRef<str>>(
		&self,
		model: M,
		node: ModelNodeId,
		info_key_utf8: S,
	) -> bool {
		let info_key_utf8 = CString::new(info_key_utf8.as_ref()).unwrap();
		unsafe {
			stereokit_sys::model_node_info_remove(
				model.as_ref().0.as_ptr(),
				node,
				info_key_utf8.as_ptr(),
			) != 0
		}
	}

	fn model_node_info_clear<M: AsRef<Model>>(&self, model: M, node: ModelNodeId) {
		unsafe { stereokit_sys::model_node_info_clear(model.as_ref().0.as_ptr(), node) }
	}

	fn model_node_info_count<M: AsRef<Model>>(&self, model: M, node: ModelNodeId) -> ModelNodeId {
		unsafe { stereokit_sys::model_node_info_count(model.as_ref().0.as_ptr(), node) }
	}

	fn model_node_info_iterate<M: AsRef<Model>>(&self, model: M, mut iterator : i32,node: ModelNodeId) -> Option<(&str, &str, i32)> {

		let out_key_utf8 = CString::new("H").unwrap().into_raw() as *mut *const std::os::raw::c_char;
		let out_value_utf8 = CString::new("H").unwrap().into_raw() as *mut *const std::os::raw::c_char;

		let ref_iterator  = &mut iterator as *mut i32;

		unsafe {
			let res = stereokit_sys::model_node_info_iterate(model.as_ref().0.as_ptr(), node, ref_iterator,  out_key_utf8,  out_value_utf8);
			if res != 0 {
				let key = CStr::from_ptr(*out_key_utf8);
				let value = CStr::from_ptr(*out_value_utf8);
				Some((key.to_str().unwrap(),value.to_str().unwrap(), *ref_iterator as i32))
			} else {None}
		}
	}

	fn sprite_find<S: AsRef<str>>(&self, id: S) -> SkResult<Sprite> {
		let cstr_id = CString::new(id.as_ref()).map_err(|_| {
			StereoKitError::SpriteFind(id.as_ref().to_string(), "CString conversion".to_string())
		})?;
		Ok(Sprite(
			NonNull::new(unsafe { stereokit_sys::sprite_find(cstr_id.as_ptr()) }).ok_or(
				StereoKitError::SpriteFind(
					id.as_ref().to_string(),
					"sprite_find failed".to_string(),
				),
			)?,
		))
	}

	fn sprite_create<S: AsRef<str>>(
		&self,
		sprite_tex: impl AsRef<Tex>,
		type_: SpriteType,
		atlas_id: S,
	) -> SkResult<Sprite> {
		let atlas_id = CString::new(atlas_id.as_ref()).unwrap();
		Ok(Sprite(
			NonNull::new(unsafe {
				stereokit_sys::sprite_create(
					sprite_tex.as_ref().0.as_ptr(),
					type_ as sprite_type_,
					atlas_id.as_ptr(),
				)
			})
			.ok_or(StereoKitError::SpriteCreate)?,
		))
	}

	fn sprite_create_file<S: AsRef<str>>(
		&self,
		file: S,
		type_: SpriteType,
		atlas_id: S,
	) -> SkResult<Sprite> {
		let atlas_id = CString::new(atlas_id.as_ref()).unwrap();
		let file = file.as_ref();
		let cfile = CString::new(file).unwrap();
		Ok(Sprite(
			NonNull::new(unsafe {
				stereokit_sys::sprite_create_file(
					cfile.as_ptr(),
					type_ as sprite_type_,
					atlas_id.as_ptr(),
				)
			})
			.ok_or(StereoKitError::SpriteFile(file.to_string()))?,
		))
	}

	fn sprite_set_id<S: AsRef<str>>(&self, sprite: impl AsRef<Sprite>, id: S) {
		let id = CString::new(id.as_ref()).unwrap();
		unsafe { stereokit_sys::sprite_set_id(sprite.as_ref().0.as_ptr(), id.as_ptr()) }
	}

	fn sprite_get_id(&self, sprite: impl AsRef<Sprite>) -> Option<&str> {
		unsafe { CStr::from_ptr(stereokit_sys::sprite_get_id(sprite.as_ref().0.as_ptr())) }
			.to_str()
			.map(|s| Some(s))
			.unwrap_or(None)
	}

	unsafe fn sprite_addref(&self, sprite: impl AsRef<Sprite>) {
		stereokit_sys::sprite_addref(sprite.as_ref().0.as_ptr())
	}

	fn sprite_release(&self, _sprite: Sprite) {}

	fn sprite_get_aspect(&self, sprite: impl AsRef<Sprite>) -> f32 {
		unsafe { stereokit_sys::sprite_get_aspect(sprite.as_ref().0.as_ptr()) }
	}

	fn sprite_get_width(&self, sprite: impl AsRef<Sprite>) -> i32 {
		unsafe { stereokit_sys::sprite_get_width(sprite.as_ref().0.as_ptr()) }
	}

	fn sprite_get_height(&self, sprite: impl AsRef<Sprite>) -> i32 {
		unsafe { stereokit_sys::sprite_get_height(sprite.as_ref().0.as_ptr()) }
	}

	fn sprite_get_dimensions_normalized(&self, sprite: impl AsRef<Sprite>) -> Vec2 {
		unsafe { stereokit_sys::sprite_get_dimensions_normalized(sprite.as_ref().0.as_ptr()) }
			.into()
	}

	fn sprite_draw(&self, sprite: impl AsRef<Sprite>, transform: impl Into<Mat4>, color: Color32) {
		let transform = transform.into().into();
		unsafe { stereokit_sys::sprite_draw(sprite.as_ref().0.as_ptr(), &transform, color) }
	}

	fn sprite_draw_at(
		&self,
		sprite: impl AsRef<Sprite>,
		transform: impl Into<Mat4>,
		anchor_position: TextAlign,
		color: Color32,
	) {
		unsafe {
			stereokit_sys::sprite_draw_at(
				sprite.as_ref().0.as_ptr(),
				transform.into().into(),
				anchor_position as text_align_,
				color,
			)
		}
	}

	fn render_set_clip(&self, near_plane: f32, far_plane: f32) {
		unsafe { stereokit_sys::render_set_clip(near_plane, far_plane) }
	}

	fn render_set_fov(&self, field_of_view_degrees: f32) {
		unsafe { stereokit_sys::render_set_fov(field_of_view_degrees) }
	}

	fn render_set_ortho_clip(&self, near_plane: f32, far_plane: f32) {
		unsafe { stereokit_sys::render_set_ortho_clip(near_plane, far_plane) }
	}

	fn render_set_ortho_size(&self, viewport_height_meters: f32) {
		unsafe { stereokit_sys::render_set_ortho_size(viewport_height_meters) }
	}

	fn render_set_projection(&self, proj: Projection) {
		unsafe { stereokit_sys::render_set_projection(proj.into()) }
	}

	fn render_get_projection(&self) -> Projection {
		unsafe { std::mem::transmute(stereokit_sys::render_get_projection()) }
	}

	fn render_get_cam_root(&self) -> Mat4 {
		unsafe { stereokit_sys::render_get_cam_root() }.into()
	}

	fn render_set_cam_root(&self, cam_root: impl Into<Mat4>) {
		let cam_root = cam_root.into().into();
		unsafe { stereokit_sys::render_set_cam_root(&cam_root) }
	}

	fn render_set_skytex(&self, sky_texture: impl AsRef<Tex>) {
		unsafe { stereokit_sys::render_set_skytex(sky_texture.as_ref().0.as_ptr()) }
	}

	fn render_set_skylight(&self, sky_light: SphericalHarmonics) {
		unsafe { stereokit_sys::render_set_skylight(&sky_light.into()) }
	}

	fn render_get_skylight(&self) -> SphericalHarmonics {
		unsafe { stereokit_sys::render_get_skylight() }.into()
	}

	fn render_set_filter(&self, layer_filter: RenderLayer) {
		unsafe { stereokit_sys::render_set_filter(layer_filter.bits as IntegerType) }
	}

	fn render_get_filter(&self) -> RenderLayer {
		unsafe { RenderLayer::from_bits_unchecked(stereokit_sys::render_get_filter() as u32) }
	}

	fn render_set_scaling(&self, display_tex_scale: f32) {
		unsafe { stereokit_sys::render_set_scaling(display_tex_scale) }
	}

	fn render_get_scaling(&self) -> f32 {
		unsafe { stereokit_sys::render_get_scaling() }
	}

	fn render_set_multisample(&self, display_tex_multisample: i32) {
		unsafe { stereokit_sys::render_set_multisample(display_tex_multisample) }
	}

	fn render_get_multisample(&self) -> i32 {
		unsafe { stereokit_sys::render_get_multisample() }
	}

	fn render_override_capture_filter(&self, use_override_filter: bool, layer_filter: RenderLayer) {
		unsafe {
			stereokit_sys::render_override_capture_filter(
				use_override_filter as bool32_t,
				layer_filter.bits as IntegerType,
			)
		}
	}

	fn render_get_capture_filter(&self) -> RenderLayer {
		unsafe {
			RenderLayer::from_bits_unchecked(stereokit_sys::render_get_capture_filter() as u32)
		}
	}

	fn render_has_capture_filter(&self) -> bool {
		unsafe { stereokit_sys::render_has_capture_filter() != 0 }
	}

	fn render_set_clear_color(&self, color_gamma: Color128) {
		unsafe { stereokit_sys::render_set_clear_color(color_gamma) }
	}

	fn render_get_clear_color(&self) -> Color128 {
		unsafe { stereokit_sys::render_get_clear_color() }
	}

	fn render_enable_skytex(&self, show_sky: bool) {
		unsafe { stereokit_sys::render_enable_skytex(show_sky as bool32_t) }
	}

	fn render_enabled_skytex(&self) -> bool {
		unsafe { stereokit_sys::render_enabled_skytex() != 0 }
	}

	fn sound_find<S: AsRef<str>>(&self, id: S) -> SkResult<Sound> {
		let c_id = CString::new(id.as_ref()).unwrap();
		Ok(Sound(
			NonNull::new(unsafe { stereokit_sys::sound_find(c_id.as_ptr()) })
				.ok_or(StereoKitError::SoundFind(id.as_ref().to_string()))?,
		))
	}

	fn sound_set_id<S: AsRef<str>>(&self, sound: impl AsRef<Sound>, id: S) {
		let id = CString::new(id.as_ref()).unwrap();
		unsafe { stereokit_sys::sound_set_id(sound.as_ref().0.as_ptr(), id.as_ptr()) }
	}

	fn sound_get_id(&self, sound: impl AsRef<Sound>) -> Option<&str> {
		unsafe { CStr::from_ptr(stereokit_sys::sound_get_id(sound.as_ref().0.as_ptr())) }
			.to_str()
			.map(|s| Some(s))
			.unwrap_or(None)
	}

	fn sound_create<S: AsRef<Path>>(&self, filename: S) -> SkResult<Sound> {
		let c_name = CString::new(filename.as_ref().to_str().unwrap()).unwrap();
		Ok(Sound(
			NonNull::new(unsafe { stereokit_sys::sound_create(c_name.as_ptr()) })
				.ok_or(StereoKitError::SoundCreate(filename.as_ref().to_path_buf()))?,
		))
	}

	fn sound_create_stream(&self, buffer: f32) -> Sound {
		Sound(NonNull::new(unsafe { stereokit_sys::sound_create_stream(buffer) }).unwrap())
	}

	fn sound_create_samples(&self, samples: &[f32]) -> Sound {
		Sound(
			NonNull::new(unsafe {
				stereokit_sys::sound_create_samples(samples.as_ptr(), samples.len() as u64)
			})
			.unwrap(),
		)
	}

	//TODO: sound_generate

	fn sound_write_samples(&self, sound: impl AsRef<Sound>, samples: &mut [f32]) {
		unsafe {
			stereokit_sys::sound_write_samples(
				sound.as_ref().0.as_ptr(),
				samples.as_ptr(),
				samples.len() as u64
			)
		}
	}

	fn sound_read_samples(&self, sound: impl AsRef<Sound>, samples: &mut [f32]) -> u64 {
		unsafe {
			stereokit_sys::sound_read_samples(
				sound.as_ref().0.as_ptr(),
				samples.as_mut_ptr(),
				samples.len() as u64,
			)
		}
	}

	fn sound_unread_samples(&self, sound: impl AsRef<Sound>) -> u64 {
		unsafe { stereokit_sys::sound_unread_samples(sound.as_ref().0.as_ptr()) }
	}

	fn sound_total_samples(&self, sound: impl AsRef<Sound>) -> u64 {
		unsafe { stereokit_sys::sound_unread_samples(sound.as_ref().0.as_ptr()) }
	}

	fn sound_cursor_samples(&self, sound: impl AsRef<Sound>) -> u64 {
		unsafe { stereokit_sys::sound_cursor_samples(sound.as_ref().0.as_ptr()) }
	}

	fn sound_play(
		&self,
		sound: impl AsRef<Sound>,
		at: impl Into<Vec3>,
		volume: f32,
	) -> SoundInstance {
		unsafe { stereokit_sys::sound_play(sound.as_ref().0.as_ptr(), at.into().into(), volume) }
	}

	fn sound_duration(&self, sound: impl AsRef<Sound>) -> f32 {
		unsafe { stereokit_sys::sound_duration(sound.as_ref().0.as_ptr()) }
	}

	unsafe fn sound_addref(&self, sound: impl AsRef<Sound>) {
		stereokit_sys::sound_addref(sound.as_ref().0.as_ptr())
	}

	fn sound_release(&self, _sound: Sound) {}

	fn sound_inst_stop(&self, sound_instance: SoundInstance) {
		unsafe { stereokit_sys::sound_inst_stop(sound_instance) }
	}

	fn sound_inst_is_playing(&self, sound_instance: SoundInstance) -> bool {
		unsafe { stereokit_sys::sound_inst_is_playing(sound_instance) != 0 }
	}

	fn sound_inst_set_pos(&self, sound_instance: SoundInstance, pos: impl Into<Vec3>) {
		unsafe { stereokit_sys::sound_inst_set_pos(sound_instance, pos.into().into()) }
	}

	fn sound_inst_set_volume(&self, sound_instance: SoundInstance, volume: f32) {
		unsafe { stereokit_sys::sound_inst_set_volume(sound_instance, volume) }
	}

	fn sound_inst_get_volume(&self, sound_instance: SoundInstance) -> f32 {
		unsafe { stereokit_sys::sound_inst_get_volume(sound_instance) }
	}

	fn mic_device_count(&self) -> i32 {
		unsafe { stereokit_sys::mic_device_count() }
	}

	fn mic_device_name(&self, index: i32) -> &str {
		unsafe { CStr::from_ptr(stereokit_sys::mic_device_name(index)) }
			.to_str()
			.unwrap()
	}

	fn mic_start<S: AsRef<str>>(&self, device_name: S) -> bool {
		let device_name = CString::new(device_name.as_ref()).unwrap();
		unsafe { stereokit_sys::mic_start(device_name.as_ptr()) != 0 }
	}

	fn mic_stop(&self) {
		unsafe { stereokit_sys::mic_stop() }
	}

	fn mic_get_stream(&self) -> Sound {
		Sound(NonNull::new(unsafe { stereokit_sys::mic_get_stream() }).unwrap())
	}

	fn mic_is_recording(&self) -> bool {
		unsafe { stereokit_sys::mic_is_recording() != 0 }
	}

	//TODO: platform_file_picker
	//TODO: platform_file_picker_sz
	//TODO: platform_file_picker_close
	//TODO: platform_file_picker_visible
	//TODO: platform_read_file
	//TODO: platform_write_file
	//TODO: platform_write_file_text
	//TODO: platform_keyboard_get_force_fallback
	//TODO: platform_keyboard_show
	//TODO: platform_keyboard_visible

	fn input_pointer_count(&self, filter: InputSource) -> i32 {
		unsafe { stereokit_sys::input_pointer_count(filter.bits as IntegerType) }
	}

	fn input_pointer(&self, index: i32, filter: InputSource) -> Pointer {
		unsafe { stereokit_sys::input_pointer(index, filter.bits as IntegerType) }.into()
	}

	fn input_hand(&self, hand: Handed) -> Hand {
		unsafe { *stereokit_sys::input_hand(hand as handed_) }.into()
	}

	//TODO: input_hand_override(hand: Handed, hand_joints: [])

	fn input_controller(&self, hand: Handed) -> Controller {
		unsafe { *stereokit_sys::input_controller(hand as handed_) }.into()
	}

	fn input_controller_menu(&self) -> ButtonState {
		unsafe { ButtonState::from_bits_unchecked(stereokit_sys::input_controller_menu() as u32) }
	}

	fn input_head(&self) -> Pose {
		unsafe { *stereokit_sys::input_head() }.into()
	}

	fn input_eyes(&self) -> Pose {
		unsafe { *stereokit_sys::input_eyes() }.into()
	}

	fn input_mouse(&self) -> Mouse {
		unsafe { *stereokit_sys::input_mouse() }.into()
	}

	fn input_key(&self, key: Key) -> ButtonState {
		unsafe { ButtonState::from_bits_unchecked(stereokit_sys::input_key(key as key_) as u32) }
	}

	fn input_text_consume(&self) -> char {
		unsafe { char::from_u32(stereokit_sys::input_text_consume()) }.unwrap()
	}

	fn input_text_reset(&self) {
		unsafe { stereokit_sys::input_text_reset() }
	}

	fn input_hand_visible(&self, hand: Handed, visible: bool) {
		unsafe { stereokit_sys::input_hand_visible(hand as handed_, visible as bool32_t) }
	}

	fn input_hand_material(&self, hand: Handed, material: Material) {
		unsafe { stereokit_sys::input_hand_material(hand as handed_, material.0.as_ptr()) }
	}

	//TODO: input_subscribe
	//TODO: input_unsubscribe

	fn input_fire_event(&self, source: InputSource, input_event: ButtonState, pointer: Pointer) {
		let pointer = pointer.into();
		unsafe {
			stereokit_sys::input_fire_event(
				source.bits as IntegerType,
				input_event.bits as IntegerType,
				&pointer,
			)
		}
	}

	fn world_has_bounds(&self) -> bool {
		unsafe { stereokit_sys::world_has_bounds() != 0 }
	}

	fn world_get_bounds_size(&self) -> Vec2 {
		unsafe { stereokit_sys::world_get_bounds_size() }.into()
	}

	fn world_get_bounds_pose(&self) -> Pose {
		unsafe { stereokit_sys::world_get_bounds_pose() }.into()
	}

	//TODO: world_from_spatial_graph
	//TODO: world_from_perception_anchor
	//TODO: world_try_from_spatial_graph
	//TODO: world_try_from_perception_anchor

	fn world_raycast(&self, ray: Ray) -> Option<Ray> {
		let ray_out = null_mut();
		match unsafe { stereokit_sys::world_raycast(ray.into(), ray_out) != 0 } {
			true => Some(unsafe { *ray_out }.into()),
			false => None,
		}
	}

	fn world_set_occlusion_enabled(&self, enabled: bool) {
		unsafe { stereokit_sys::world_set_occlusion_enabled(enabled as bool32_t) }
	}

	fn world_get_occlusion_enabled(&self) -> bool {
		unsafe { stereokit_sys::world_get_occlusion_enabled() != 0 }
	}

	fn world_set_raycast_enabled(&self, enabled: bool) {
		unsafe { stereokit_sys::world_set_raycast_enabled(enabled as bool32_t) }
	}

	fn world_get_raycast_enabled(&self) -> bool {
		unsafe { stereokit_sys::world_get_raycast_enabled() != 0 }
	}

	fn world_set_occlusion_material(&self, material: impl AsRef<Material>) {
		unsafe { stereokit_sys::world_set_occlusion_material(material.as_ref().0.as_ptr()) }
	}

	fn world_get_occlusion_material(&self) -> Material {
		Material(NonNull::new(unsafe { stereokit_sys::world_get_occlusion_material() }).unwrap())
	}

	fn world_set_refresh_type(&self, refresh_type: WorldRefresh) {
		unsafe { stereokit_sys::world_set_refresh_type(refresh_type as world_refresh_) }
	}

	fn world_get_refresh_type(&self) -> WorldRefresh {
		unsafe { std::mem::transmute(stereokit_sys::world_get_refresh_type()) }
	}

	fn world_set_refresh_radius(&self, radius_meters: f32) {
		unsafe { stereokit_sys::world_set_refresh_radius(radius_meters) }
	}

	fn world_get_refresh_radius(&self) -> f32 {
		unsafe { stereokit_sys::world_get_refresh_radius() }
	}

	fn world_get_refresh_interval(&self) -> f32 {
		unsafe { stereokit_sys::world_get_refresh_interval() }
	}

	fn backend_xr_get_type(&self) -> BackendXrType {
		unsafe { std::mem::transmute(stereokit_sys::backend_xr_get_type()) }
	}

	fn backend_openxr_get_instance(&self) -> OpenXrHandle {
		unsafe { stereokit_sys::backend_openxr_get_instance() }
	}

	fn backend_openxr_get_session(&self) -> OpenXrHandle {
		unsafe { stereokit_sys::backend_openxr_get_session() }
	}

	fn backend_openxr_get_system_id(&self) -> OpenXrHandle {
		unsafe { stereokit_sys::backend_openxr_get_system_id() }
	}

	fn backend_openxr_get_space(&self) -> OpenXrHandle {
		unsafe { stereokit_sys::backend_openxr_get_space() }
	}

	fn backend_openxr_get_function<S: AsRef<str>>(
		&self,
		function_name: S,
	) -> *mut std::os::raw::c_void {
		let function_name = CString::new(function_name.as_ref()).unwrap();
		unsafe { stereokit_sys::backend_openxr_get_function(function_name.as_ptr()) }
	}

	fn backend_openxr_ext_enabled<S: AsRef<str>>(&self, extension_name: S) -> bool {
		let extension_name = CString::new(extension_name.as_ref()).unwrap();
		unsafe { stereokit_sys::backend_openxr_ext_enabled(extension_name.as_ptr()) != 0 }
	}

	fn backend_openxr_ext_request<S: AsRef<str>>(&self, extension_name: S) {
		let extension_name = CString::new(extension_name.as_ref()).unwrap();
		unsafe { stereokit_sys::backend_openxr_ext_request(extension_name.as_ptr()) }
	}

	fn backend_openxr_use_minimum_exts(&self, use_minimum_exts: bool) {
		unsafe { stereokit_sys::backend_openxr_use_minimum_exts(use_minimum_exts as bool32_t) }
	}

	//TODO: backend_openxr_composition_layer
	//TODO: backend_openxr_add_callback_pre_session_create
	//TODO: backend_openxr_add_callback_poll_event
	//TODO: backend_openxr_remove_callback_poll_event

	fn backend_platform_get(&self) -> BackendPlatform {
		unsafe { std::mem::transmute(stereokit_sys::backend_platform_get()) }
	}

	fn backend_android_get_java_jvm(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_android_get_java_vm() }
	}

	fn backend_android_get_activity(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_android_get_activity() }
	}

	fn backend_android_get_jni_env(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_android_get_jni_env() }
	}

	fn backend_d3d11_get_d3d_device(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_d3d11_get_d3d_device() }
	}

	fn backend_d3d11_get_d3d_context(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_d3d11_get_d3d_context() }
	}

	fn backend_opengl_wgl_get_hdc(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_d3d11_get_d3d_context() }
	}

	fn backend_opengl_wgl_get_hglrc(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_opengl_wgl_get_hglrc() }
	}

	fn backend_opengl_glx_get_context(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_opengl_glx_get_context() }
	}

	fn backend_opengl_glx_get_display(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_opengl_glx_get_display() }
	}

	fn backend_opengl_glx_get_drawable(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_opengl_glx_get_drawable() }
	}

	fn backend_opengl_egl_get_context(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_opengl_egl_get_context() }
	}

	fn backend_opengl_egl_get_config(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_opengl_egl_get_config() }
	}

	fn backend_opengl_egl_get_display(&self) -> *mut std::os::raw::c_void {
		unsafe { stereokit_sys::backend_opengl_egl_get_display() }
	}

	fn log_diag<S: AsRef<str>>(&self, text: S) {
		let text = CString::new(text.as_ref()).unwrap();
		unsafe { stereokit_sys::log_diag(text.as_ptr()) }
	}

	fn log_info<S: AsRef<str>>(&self, text: S) {
		let text = CString::new(text.as_ref()).unwrap();
		unsafe { stereokit_sys::log_info(text.as_ptr()) }
	}

	fn log_warn<S: AsRef<str>>(&self, text: S) {
		let text = CString::new(text.as_ref()).unwrap();
		unsafe { stereokit_sys::log_warn(text.as_ptr()) }
	}

	fn log_err<S: AsRef<str>>(&self, text: S) {
		let text = CString::new(text.as_ref()).unwrap();
		unsafe { stereokit_sys::log_err(text.as_ptr()) }
	}

	fn log_write<S: AsRef<str>>(&self, level: LogLevel, text: S) {
		let text = CString::new(text.as_ref()).unwrap();
		unsafe { stereokit_sys::log_write(level as log_, text.as_ptr()) }
	}

	fn log_set_filter(&self, level: LogLevel) {
		unsafe { stereokit_sys::log_set_filter(level as log_) }
	}

	fn log_set_colors(&self, colors: LogColors) {
		unsafe { stereokit_sys::log_set_colors(colors as log_colors_) }
	}

	//TODO: log_subscribe
	//TODO: log_unsubscribe

	//TODO: idk assets_releaseref_threadsafe

	fn assets_releaseref_threadsafe(&self, asset: Asset) {
		unsafe { stereokit_sys::assets_releaseref_threadsafe(asset.0.as_ptr()) }
	}

	fn assets_current_task(&self) -> i32 {
		unsafe { stereokit_sys::assets_current_task() }
	}

	fn assets_total_tasks(&self) -> i32 {
		unsafe { stereokit_sys::assets_total_tasks() }
	}

	fn assets_current_task_priority(&self) -> i32 {
		unsafe { stereokit_sys::assets_current_task_priority() }
	}

	fn assets_block_for_priority(&self, priority: i32) {
		unsafe { stereokit_sys::assets_block_for_priority(priority) }
	}

	fn assets_count(&self) -> i32 {
		unsafe { stereokit_sys::assets_count() }
	}

	fn assets_get_index(&self, index: i32) -> Option<Asset> {
		Some(Asset(NonNull::new(unsafe {
			stereokit_sys::assets_get_index(index)
		})?))
	}

	fn assets_get_type(&self, index: i32) -> AssetType {
		unsafe { std::mem::transmute(stereokit_sys::assets_get_type(index)) }
	}

	fn asset_set_id(&self, asset: impl AsRef<Asset>, id: impl AsRef<str>) {
		let id = CString::new(id.as_ref()).unwrap();
		unsafe { stereokit_sys::asset_set_id(asset.as_ref().0.as_ptr(), id.as_ptr()) }
	}

	fn asset_get_id(&self, asset: impl AsRef<Asset>) -> &str {
		unsafe { CStr::from_ptr(stereokit_sys::asset_get_id(asset.as_ref().0.as_ptr())) }
			.to_str()
			.unwrap()
	}

	fn window<S: AsRef<str>>(
		&self,
		window_title: S,
		mut pose: impl AsMut<Pose>,
		size: impl Into<Vec2>,
		window_type: WindowType,
		move_type: MoveType,
		content_closure: impl FnOnce(&WindowContext),
	) {
		let window_title = CString::new(window_title.as_ref()).unwrap();
		let pose = pose.as_mut();
		let mut pose_2: pose_t = pose.clone().into();
		let size = size.into();
		unsafe {
			stereokit_sys::ui_window_begin(
				window_title.as_ptr(),
				&mut pose_2 as *mut pose_t,
				size.into(),
				window_type as ui_win_,
				move_type as ui_move_,
			)
		}

		let context = WindowContext(PhantomData);
		#[cfg(feature = "auto-hash-id-location")]
		unsafe {
			context.new_locations();
		}

		content_closure(&WindowContext(PhantomData));

		#[cfg(feature = "auto-hash-id-location")]
		unsafe {
			context.new_locations();
		}

		*pose = pose_2.into();
		unsafe {
			stereokit_sys::ui_window_end();
		}
	}
}

pub fn ray_intersect_plane<V3: Into<Vec3>>(
	ray: Ray,
	plane_pt: V3,
	plane_normal: V3,
) -> Option<f32> {
	let mut out_t = 0.0;
	match unsafe {
		stereokit_sys::ray_intersect_plane(
			ray.into(),
			plane_pt.into().into(),
			plane_normal.into().into(),
			&mut out_t,
		) != 0
	} {
		true => Some(out_t),
		false => None,
	}
}

pub fn ray_from_mouse(screen_pixel_pos: impl Into<Vec2>) -> Option<Ray> {
	let mut ray: ray_t = Ray::default().into();
	match unsafe { stereokit_sys::ray_from_mouse(screen_pixel_pos.into().into(), &mut ray) != 0 } {
		true => Some(ray.into()),
		false => None,
	}
}

/// Creates a [`Plane`] from 3 points that are directly on that [`Plane`].
/// * `p1` - First point on the plane
/// * `p2` - Second point on the plane
/// * `p3` - Third point on the plane
pub fn plane_from_points<V3: Into<Vec3>>(p1: V3, p2: V3, p3: V3) -> Plane {
	unsafe {
		stereokit_sys::plane_from_points(p1.into().into(), p2.into().into(), p3.into().into())
	}
	.into()
}

pub fn plane_from_ray(ray: Ray) -> Plane {
	unsafe { stereokit_sys::plane_from_ray(ray.into()) }.into()
}

/// Checks the intersection of this ray with a [`Plane`]!
/// * `plane` - Any plane you want to intersect with.
/// * `at` - An out parameter that will hold the intersection
/// point. If there's no intersection, this will be (0,0,0).
///
/// `returns` - True if there's an intersection, false if not. Refer to
/// the `at` parameter for intersection information!
pub fn plane_ray_intersect(plane: Plane, ray: Ray) -> Option<Vec3> {
	let mut point = Vec3::default().into();
	match unsafe { stereokit_sys::plane_ray_intersect(plane.into(), ray.into(), &mut point) != 0 } {
		true => Some(point.into()),
		false => None,
	}
}

pub fn plane_line_intersect<V3: Into<Vec3>>(plane: Plane, p1: V3, p2: V3) -> Option<Vec3> {
	let mut point = Vec3::default().into();
	match unsafe {
		stereokit_sys::plane_line_intersect(
			plane.into(),
			p1.into().into(),
			p2.into().into(),
			&mut point,
		) != 0
	} {
		true => Some(point.into()),
		false => None,
	}
}

/// Finds the closest point on this [`Plane`] to the given point!
/// * `plane` - The [`Plane`]
/// * `pt` - The point you have that's not necessarily on the [`Plane`].
///
/// `returns` - The point on the plane that's closest to the `pt` parameter.
pub fn plane_point_closest<V3: Into<Vec3>>(plane: Plane, pt: V3) -> Vec3 {
	unsafe { stereokit_sys::plane_point_closest(plane.into(), pt.into().into()) }.into()
}

/// Intersects a [`Ray`] with this [`Sphere`], and finds if they intersect,
/// and if so, where that intersection is! This only finds the closest
/// intersection point to the origin of the [`Ray`].
/// * `sphere` - A [`Sphere`] to intersect with.
/// * `ray` - A [`Ray`] to intersect with.
///
/// `returns` - [`Vec3`] if intersection occurs, [`None`] if it doesn't.
pub fn sphere_ray_intersect(sphere: Sphere, ray: Ray) -> Option<Vec3> {
	let mut pt = Vec3::default().into();
	match unsafe { stereokit_sys::sphere_ray_intersect(sphere.into(), ray.into(), &mut pt) != 0 } {
		true => Some(pt.into()),
		false => None,
	}
}

pub fn sphere_point_contains<V3: Into<Vec3>>(sphere: Sphere, pt: V3) -> bool {
	unsafe { stereokit_sys::sphere_point_contains(sphere.into(), pt.into().into()) != 0 }
}

pub fn bounds_ray_intersect(bounds: Bounds, ray: Ray) -> Option<Vec3> {
	let mut pt = Vec3::default().into();
	match unsafe { stereokit_sys::bounds_ray_intersect(bounds.into(), ray.into(), &mut pt) != 0 } {
		true => Some(pt.into()),
		false => None,
	}
}

pub fn bounds_point_contains<V3: Into<Vec3>>(bounds: Bounds, pt: V3) -> bool {
	unsafe { stereokit_sys::bounds_point_contains(bounds.into(), pt.into().into()) != 0 }
}

pub fn bounds_line_contains<V3: Into<Vec3>>(bounds: Bounds, pt1: V3, pt2: V3) -> bool {
	unsafe {
		stereokit_sys::bounds_line_contains(bounds.into(), pt1.into().into(), pt2.into().into())
			!= 0
	}
}

pub fn bounds_capsule_contains<V3: Into<Vec3>>(
	bounds: Bounds,
	pt1: V3,
	pt2: V3,
	radius: f32,
) -> bool {
	unsafe {
		stereokit_sys::bounds_capsule_contains(
			bounds.into(),
			pt1.into().into(),
			pt2.into().into(),
			radius,
		) != 0
	}
}

pub fn bounds_grow_to_fit_pt<V3: Into<Vec3>>(bounds: Bounds, pt: V3) -> Bounds {
	unsafe { stereokit_sys::bounds_grow_to_fit_pt(bounds.into(), pt.into().into()).into() }
}

pub fn bounds_grow_to_fit_box(
	bounds: Bounds,
	box_: Bounds,
	opt_box_transform: Option<impl Into<glam::Mat4>>,
) -> Bounds {
	unsafe {
		stereokit_sys::bounds_grow_to_fit_box(
			bounds.into(),
			box_.into(),
			match opt_box_transform {
				None => null(),
				Some(transform) => &transform.into().into(),
			},
		)
		.into()
	}
}

pub fn bounds_transform(bounds: Bounds, transform: impl Into<Mat4>) -> Bounds {
	unsafe { stereokit_sys::bounds_transform(bounds.into(), transform.into().into()).into() }
}

pub fn ray_point_closest(ray: Ray, pt: impl Into<Vec3>) -> Vec3 {
	unsafe { stereokit_sys::ray_point_closest(ray.into(), pt.into().into()).into() }
}

pub fn model_ray_intersect<M: AsRef<Model>>(
	model: M,
	ray: Ray,
	cull_mode: CullMode,
) -> Option<Ray> {
	let ray_ptr = null_mut();
	match unsafe {
		stereokit_sys::model_ray_intersect(
			model.as_ref().0.as_ptr(),
			ray.into(),
			ray_ptr,
			std::mem::transmute(cull_mode),
		) != 0
	} {
		true => Some(unsafe { std::mem::transmute(*ray_ptr) }),
		false => None,
	}
}

pub fn model_ray_intersect_bhv<M: AsRef<Model>>(
	model: M,
	ray: Ray,
	cull_mode: CullMode,
) -> Option<Ray> {
	let ray_ptr = null_mut();
	match unsafe {
		stereokit_sys::model_ray_intersect_bvh(
			model.as_ref().0.as_ptr(),
			ray.into(),
			ray_ptr,
			std::mem::transmute(cull_mode),
		) != 0
	} {
		true => Some(unsafe { std::mem::transmute(*ray_ptr) }),
		false => None,
	}
}

//TODO: model_ray_intersect_bvh_detailed

/// Creates a Red/Green/Blue gamma space color from
/// Hue/Saturation/Value information.
///
/// * `hue` - Hue most directly relates to the color as we
/// think of it! 0 is red, 0.1667 is yellow, 0.3333 is green, 0.5 is
/// cyan, 0.6667 is blue, 0.8333 is magenta, and 1 is red again!
///
/// * `saturation` - The vibrancy of the color, where 0 is
/// straight up a shade of gray, and 1 is 'poke you in the eye
/// colorful'
///
/// * `value` - The brightness of the color! 0 is always
/// black.
///
/// * `transparency` - Also known as alpha! This is does not
/// affect the rgb components of the resulting color, it'll just get
/// slotted into the colors opacity value.
///
/// `returns` - A gamma space RGB color!
pub fn color_hsv(hue: f32, saturation: f32, value: f32, transparency: f32) -> Color128 {
	unsafe { stereokit_sys::color_hsv(hue, saturation, value, transparency) }
}

/// Converts the gamma space color to a Hue/Saturation/Value
/// format! Does not consider transparency when calculating the
/// result.
///
/// `returns` - Hue, Saturation, and Value, stored in x, y, and z
/// respectively. All values are between 0-1.
pub fn color_to_hsv(color: Color128) -> Vec3 {
	unsafe { stereokit_sys::color_to_hsv(&color).into() }
}

/// Creates a gamma space RGB color from a CIE-L*ab color
/// space. CIE-L*ab is a color space that models human perception,
/// and has significantly more accurate to perception lightness
/// values, so this is an excellent color space for color operations
/// that wish to preserve color brightness properly.
///
/// Traditionally, values are L \[0,100\], a,b \[-200,+200\] but here we
/// normalize them all to the 0-1 range. If you hate it, let me know
/// why!
///
/// * `l` - Lightness of the color! Range is 0-1.
///
/// * `a` - is from red to green. Range is 0-1.
///
/// * `b` - is from blue to yellow. Range is 0-1.
/// * `transparency` - The opacity copied into the final color!
///
/// `returns` - A gamma space RGBA color constructed from the LAB
/// values.
pub fn color_lab(l: f32, a: f32, b: f32, transparency: f32) -> Color128 {
	unsafe { stereokit_sys::color_lab(l, a, b, transparency) }
}

/// Converts the gamma space RGB color to a CIE LAB color
/// space value! Conversion back and forth from LAB space could be
/// somewhat lossy.
///
/// `returns` - An LAB vector where x=L, y=A, z=B.
pub fn color_to_lab(color: Color128) -> Vec3 {
	unsafe { stereokit_sys::color_to_lab(&color) }.into()
}

/// Converts this from a gamma space color, into a linear
/// space color! If this is not a gamma space color, this will just
/// make your color wacky!
///
/// `returns` - A linear space color.
pub fn color_to_linear(srgb_gamma_correct: Color128) -> Color128 {
	unsafe { stereokit_sys::color_to_linear(srgb_gamma_correct) }
}

/// Converts this from a linear space color, into a gamma
/// space color! If this is not a linear space color, this will just
/// make your color wacky!
///
/// `returns` - A gamma space color.
pub fn color_to_gamma(srgb_linear: Color128) -> Color128 {
	unsafe { stereokit_sys::color_to_gamma(srgb_linear) }
}

/// Creates a new, completely empty gradient.
pub fn gradient_create() -> Gradient {
	Gradient(NonNull::new(unsafe { stereokit_sys::gradient_create() }).unwrap())
}

/// Creates a new gradient from the list of color keys!
///
/// * `keys` - These can be in any order that you like, they’ll be sorted by their GradientKey.position value regardless!
pub fn gradient_create_keys(keys: &[GradientKey]) -> Gradient {
	Gradient(
		NonNull::new(unsafe {
			stereokit_sys::gradient_create_keys(
				keys.iter()
					.map(|a| a.clone().into())
					.collect::<Vec<_>>()
					.as_ptr(),
				keys.len() as i32,
			)
		})
		.unwrap(),
	)
}

/// This adds a color key into the list. It’ll get inserted to the right slot based on its position.
pub fn gradient_add(gradient: impl AsRef<Gradient>, color_linear: Color128, position: f32) {
	unsafe { stereokit_sys::gradient_add(gradient.as_ref().0.as_ptr(), color_linear, position) }
}

/// Updates the color key at the given index! This will NOT re-order color keys if they are moved past another key’s position, which could lead to strange behavior.
pub fn gradient_set(
	gradient: impl AsRef<Gradient>,
	index: i32,
	color_linear: Color128,
	position: f32,
) {
	unsafe {
		stereokit_sys::gradient_set(gradient.as_ref().0.as_ptr(), index, color_linear, position)
	}
}

/// Removes the color key at the given index!
pub fn gradient_remove(gradient: impl AsRef<Gradient>, index: i32) {
	unsafe { stereokit_sys::gradient_remove(gradient.as_ref().0.as_ptr(), index) }
}

/// The number of color keys present in this gradient.
pub fn gradient_count(gradient: impl AsRef<Gradient>) -> i32 {
	unsafe { stereokit_sys::gradient_count(gradient.as_ref().0.as_ptr()) }
}

/// Samples the gradient’s color at the given position!
pub fn gradient_get(gradient: impl AsRef<Gradient>, at: f32) -> Color128 {
	unsafe { stereokit_sys::gradient_get(gradient.as_ref().0.as_ptr(), at) }
}

/// Samples the gradient’s color at the given position, and converts it to a 32 bit color. If your RGBA color values are outside of the 0-1 range, then you’ll get some issues as they’re converted to 0-255 range bytes!
pub fn gradient_get32(gradient: impl AsRef<Gradient>, at: f32) -> Color32 {
	unsafe { stereokit_sys::gradient_get32(gradient.as_ref().0.as_ptr(), at) }
}

/// Releases the asset, automatically called on drop.
pub fn gradient_release(gradient: &mut Gradient) {
	unsafe { stereokit_sys::gradient_release(gradient.0.as_ptr()) }
}

/// Creates a SphericalHarmonic from an array of coefficients. Useful for loading stored data!
pub fn sh_create(lights: &[ShLight]) -> SphericalHarmonics {
	unsafe {
		stereokit_sys::sh_create(
			lights
				.iter()
				.map(|a| a.clone().into())
				.collect::<Vec<_>>()
				.as_ptr(),
			lights.len() as i32,
		)
	}
	.into()
}

/// Scales all the SphericalHarmonic’s coefficients! This behaves as if you’re modifying the brightness of the lighting this object represents.
pub fn sh_brightness(ref_harmonics: &mut SphericalHarmonics, scale: f32) {
	unsafe { stereokit_sys::sh_brightness(std::mem::transmute(ref_harmonics), scale) }
}

/// Adds a ‘directional light’ to the lighting approximation. This can be used to bake a multiple light setup, or accumulate light from a field of points.
pub fn sh_add(ref_harmonics: &mut SphericalHarmonics, light_dir: Vec3, light_color: Vec3) {
	unsafe {
		stereokit_sys::sh_add(
			std::mem::transmute(ref_harmonics),
			light_dir.into(),
			light_color.into(),
		)
	}
}

///Look up the color information in a particular direction!
pub fn sh_lookup(harmonics: &SphericalHarmonics, normal: Vec3) -> Color128 {
	unsafe { stereokit_sys::sh_lookup(&(*harmonics).into(), normal.into()) }
}

///Returns the dominant direction of the light represented by this spherical harmonics data. The direction value is normalized. You can get the color of the light in this direction by using the struct’s Sample method: light.Sample(-light.DominantLightDirection).
pub fn sh_dominant_dir(harmonics: &SphericalHarmonics) -> Vec3 {
	unsafe { stereokit_sys::sh_dominant_dir(&(*harmonics).into()) }.into()
}

/// Releases the asset, automatically called on drop.
fn mesh_release(mesh: &mut Mesh) {
	unsafe { stereokit_sys::mesh_release(mesh.0.as_ptr()) }
}

fn font_release(font: &mut Font) {
	unsafe { stereokit_sys::font_release(font.0.as_ptr()) }
}

fn shader_release(shader: &mut Shader) {
	unsafe { stereokit_sys::shader_release(shader.0.as_ptr()) }
}

fn material_release(material: &mut Material) {
	unsafe { stereokit_sys::material_release(material.0.as_ptr()) }
}

fn material_buffer_release(material_buffer: &mut MaterialBuffer) {
	unsafe { stereokit_sys::material_buffer_release(material_buffer.0.as_ptr()) }
}

fn model_release(model: &mut Model) {
	unsafe { stereokit_sys::model_release(model.0.as_ptr()) }
}

fn sprite_release(sprite: &mut Sprite) {
	unsafe { stereokit_sys::sprite_release(sprite.0.as_ptr()) }
}

fn sound_release(sound: &mut Sound) {
	unsafe { stereokit_sys::sound_release(sound.0.as_ptr()) }
}

pub struct WindowContext(PhantomData<*const ()>);


#[cfg(feature = "auto-hash-id-location")]
static mut LOCATIONS: Option<HashSet<u64>> = None;

impl WindowContext {
	pub unsafe fn create_unsafe() -> Self {
		Self { 0: Default::default() }
	}

	#[cfg(feature = "auto-hash-id-location")]
	pub unsafe fn get_locations(&self) -> &'static mut HashSet<u64> {
		LOCATIONS.as_mut().unwrap()
	}
	#[cfg(feature = "auto-hash-id-location")]
	unsafe fn new_locations(&self) {
		LOCATIONS.replace(HashSet::with_capacity(10000));
	}

	pub fn push_text_style(&self, style: TextStyle) {
		unsafe { stereokit_sys::ui_push_text_style(style.0) }
	}
	pub fn pop_text_style(&self) {
		unsafe {
			stereokit_sys::ui_pop_text_style();
		}
	}
	pub fn text_style(&self, style: TextStyle, content_closure: impl FnOnce(&WindowContext)) {
		self.push_text_style(style);
		content_closure(self);
		self.pop_text_style();
	}
	pub fn push_tint(&self, tint_gamma: Color128) {
		unsafe {
			stereokit_sys::ui_push_tint(tint_gamma);
		}
	}
	pub fn pop_tint(&self) {
		unsafe {
			stereokit_sys::ui_pop_tint();
		}
	}
	pub fn tint(&self, tint_gamma: Color128, content_closure: impl FnOnce(&WindowContext)) {
		self.push_tint(tint_gamma);
		content_closure(self);
		self.pop_tint();
	}
	pub fn push_enabled(&self, enabled: bool) {
		unsafe {
			stereokit_sys::ui_push_enabled(enabled as bool32_t);
		}
	}
	pub fn pop_enabled(&self) {
		unsafe {
			stereokit_sys::ui_pop_enabled();
		}
	}
	pub fn enabled(&self, enabled: bool, content_closure: impl FnOnce(&WindowContext)) {
		self.push_enabled(enabled);
		content_closure(self);
		self.pop_enabled();
	}
	pub fn push_preserve_keyboard(&self, preserve_keyboard: bool) {
		unsafe { stereokit_sys::ui_push_preserve_keyboard(preserve_keyboard as bool32_t) }
	}
	pub fn pop_preserve_keyboard(&self) {
		unsafe {
			stereokit_sys::ui_pop_preserve_keyboard();
		}
	}
	pub fn preserve_keyboard(
		&self,
		preserve_keyboard: bool,
		content_closure: impl FnOnce(&WindowContext),
	) {
		self.push_preserve_keyboard(preserve_keyboard);
		content_closure(self);
		self.pop_preserve_keyboard();
	}
	pub fn push_surface(
		&self,
		surface_pose: Pose,
		layout_state: impl Into<Vec3>,
		layout_dimensions: impl Into<Vec2>,
	) {
		unsafe {
			stereokit_sys::ui_push_surface(
				surface_pose.into(),
				layout_state.into().into(),
				layout_dimensions.into().into(),
			)
		}
	}
	pub fn pop_surface(&self) {
		unsafe { stereokit_sys::ui_pop_surface() }
	}
	pub fn surface(
		&self,
		surface_pose: Pose,
		layout_state: impl Into<Vec3>,
		layout_dimensions: impl Into<Vec2>,
		content_closure: impl FnOnce(&WindowContext),
	) {
		self.push_surface(surface_pose, layout_state, layout_dimensions);
		content_closure(self);
		self.pop_surface();
	}
	pub fn push_cut_layout(
		&self,
		ui_cut: UiCut,
		size: f32,
		add_margin: bool,
	) {
		unsafe {
			stereokit_sys::ui_layout_push_cut(
				ui_cut as ui_cut_,
				size,
				add_margin as bool32_t
			)
		}
	}
	pub fn pop_layout(&self) {
		unsafe {
			stereokit_sys::ui_layout_pop()
		}
	}
	pub fn cut_layout(&self, ui_cut: UiCut, size: f32, add_margin: bool, content_closure: impl FnOnce(&WindowContext)) {
		self.push_cut_layout(ui_cut, size, add_margin);
		content_closure(self);
		self.pop_layout();
	}
	pub fn push_id(&self, id: impl AsRef<str>) -> u64 {
		let id = CString::new(id.as_ref()).unwrap();
		unsafe { stereokit_sys::ui_push_id(id.as_ptr()) }
	}
	pub fn push_idi(&self, id: i32) -> u64 {
		unsafe { stereokit_sys::ui_push_idi(id) }
	}
	pub fn pop_id(&self) {
		unsafe { stereokit_sys::ui_pop_id() }
	}
	pub fn id(&self, id: impl AsRef<str>, content_closure: impl FnOnce(&WindowContext, u64)) {
		let id = self.push_id(id);
		content_closure(self, id);
		self.pop_id();
	}
	pub fn idi(&self, id: i32, content_closure: impl FnOnce(&WindowContext, u64)) {
		let id = self.push_idi(id);
		content_closure(self, id);
		self.pop_id();
	}
	pub fn stack_hash(&self, id: impl AsRef<str>) -> u64 {
		let id = CString::new(id.as_ref()).unwrap();
		unsafe {
			stereokit_sys::ui_stack_hash(id.as_ptr())
		}
	}
	pub fn label(&self, text: impl AsRef<str>, use_padding: bool) {
		let c_str = std::ffi::CString::new(text.as_ref()).unwrap();
		unsafe {
			stereokit_sys::ui_label(c_str.as_ptr(), use_padding as bool32_t);
		}
	}
	pub fn toggle(&self, text: impl AsRef<str>, pressed: &mut bool) {
		let c_str = std::ffi::CString::new(text.as_ref()).unwrap();
		unsafe {
			stereokit_sys::ui_toggle(c_str.as_ptr(), pressed as &mut _ as *mut _ as *mut i32);
		}
	}
	pub fn button(&self, text: impl AsRef<str>) -> bool {
		let c_str = std::ffi::CString::new(text.as_ref()).unwrap();
		unsafe {
			stereokit_sys::ui_button(c_str.as_ptr()) != 0
		}
	}
	pub fn button_at(&self, text: impl AsRef<str>, window_relative_pos: impl Into<Vec3>, size: impl Into<Vec2>) -> bool {
		let c_str = std::ffi::CString::new(text.as_ref()).unwrap();
		unsafe {
			stereokit_sys::ui_button_at(c_str.as_ptr(), window_relative_pos.into().into(), size.into().into()) != 0
		}
	}
	pub fn button_img(&self,text: impl AsRef<str>, sprite: &mut Sprite, image_layout: UiBtnLayout) -> bool {
		let c_str = std::ffi::CString::new(text.as_ref()).unwrap();

		unsafe {
			stereokit_sys::ui_button_img(c_str.as_ptr() as *const i8, sprite.0.as_mut(), image_layout as ui_btn_layout_) != 0
		}
	}
	pub fn button_img_size(&self,text: impl AsRef<str>, sprite: &mut Sprite, image_layout: UiBtnLayout, size: impl Into<Vec2>) -> bool {
		let c_str = std::ffi::CString::new(text.as_ref()).unwrap();

		unsafe {
			stereokit_sys::ui_button_img_sz(c_str.as_ptr() as *const i8, sprite.0.as_mut(), image_layout as ui_btn_layout_, size.into().into()) != 0
		}
	}
	pub fn button_img_at(&self,text: impl AsRef<str>, sprite: &mut Sprite, image_layout: UiBtnLayout, window_relative_pos: impl Into<Vec3>, size: impl Into<Vec2>) -> bool {
		let c_str = std::ffi::CString::new(text.as_ref()).unwrap();

		unsafe {
			stereokit_sys::ui_button_img_at(c_str.as_ptr() as *const i8, sprite.0.as_mut(), image_layout as ui_btn_layout_,window_relative_pos.into().into(), size.into().into()) != 0
		}
	}
	pub fn same_line(&self) {
		unsafe {
			stereokit_sys::ui_sameline()
		}
	}
	pub fn hz_slider(&self, id: impl AsRef<str>, value: &mut f32, min: f32, max: f32, step: f32, width: f32) {
		let c_str = std::ffi::CString::new(id.as_ref()).unwrap();
		unsafe {
			stereokit_sys::ui_hslider(c_str.as_ptr(), value as *mut f32, min, max, step, width, 0, 0);
		}
	}
	pub fn set_color(&self, color: Color128) {
		unsafe {
			stereokit_sys::ui_set_color(color)
		}
	}
	pub fn set_theme_color(&self, color_type: UiColor, color_gamma: Color128) {
		unsafe {
			stereokit_sys::ui_set_theme_color(color_type as ui_color_, color_gamma);
		}
	}
	pub fn area_remaining(&self) -> Vec2 {
		unsafe {
			stereokit_sys::ui_area_remaining()
		}.into()
	}
	pub fn layout_at(&self) -> Vec3 {
		unsafe {
			stereokit_sys::ui_layout_at()
		}.into()
	}
	pub fn image(&self, image: &mut Sprite, size: impl Into<Vec2>) {
		unsafe {
			stereokit_sys::ui_image(image.0.as_mut(), size.into().into());
		}
	}
}
