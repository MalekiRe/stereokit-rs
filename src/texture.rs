#![allow(non_upper_case_globals)]

use crate::lifecycle::StereoKitInstanceWrapper;
use crate::render::SphericalHarmonics;
use crate::values::{
	color128_from, color128_to, color32_from, color32_to, Color128, Color32, Vec3, IntType,
};
use crate::StereoKit;
use bitflags::bitflags;
use num_enum::TryFromPrimitive;
use std::ffi::{c_void, CString};
use std::fmt::Error;
use std::mem::transmute;
use std::path::Path;
use std::ptr::NonNull;
use std::rc::{Rc, Weak};
use stereokit_sys::{_gradient_t, _tex_t, bool32_t, gradient_t, spherical_harmonics_t, vec3};
use ustr::ustr;

/// What type of color information will the texture contain? A
/// good default here is Rgba32.
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
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
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
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
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
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

bitflags! {
	/// Textures come in various types and flavors! These are bit-flags
	/// that tell StereoKit what type of texture we want; and how the application
	/// might use it!
	pub struct TextureType: u32 {
		/// A standard color image; without any generated mip-maps.
		const ImageNoMips = 1 << 0;
		/// A size sided texture that's used for things like skyboxes;
		/// environment maps; and reflection probes. It behaves like a texture
		/// array with 6 textures.
		const Cubemap = 1 << 1;
		/// This texture can be rendered to! This is great for textures
		/// that might be passed in as a target to Renderer.Blit; or other
		/// such situations.
		const RenderTarget = 1 << 2;
		/// This texture contains depth data; not color data!
		const Depth = 1 << 3;
		/// This texture will generate mip-maps any time the contents
		/// change. Mip-maps are a list of textures that are each half the
		/// size of the one before them! This is used to prevent textures from
		/// 'sparkling' or aliasing in the distance.
		const Mips = 1 << 4;
		/// This texture's data will be updated frequently from the
		/// CPU (not renders)! This ensures the graphics card stores it
		/// someplace where writes are easy to do quickly.
		const Dynamic = 1 << 5;
		/// A standard color image that also generates mip-maps
		/// automatically.
		const Image = Self::ImageNoMips.bits | Self::Mips.bits;
	}
}

pub struct Gradient {
	gradient: gradient_t,
}
impl Gradient {
	pub fn new(stereokit: &StereoKit) -> Self {
		Gradient {
			gradient: unsafe { stereokit_sys::gradient_create() },
		}
	}
	pub fn add(&mut self, color: Color128, position: f32) {
		unsafe { stereokit_sys::gradient_add(self.gradient, std::mem::transmute(color), position) }
	}
}

pub struct Texture {
	sk: StereoKitInstanceWrapper,
	pub(super) tex: NonNull<_tex_t>,
}
impl Texture {
	pub fn from_file(
		sk: &StereoKit,
		file_path: impl AsRef<Path>,
		is_srgb: bool,
		priority: i32,
	) -> Option<Self> {
		let file_path = ustr(file_path.as_ref().as_os_str().to_str()?);
		Some(Texture {
			sk: sk.get_wrapper(),
			tex: NonNull::new(unsafe {
				stereokit_sys::tex_create_file(file_path.as_char_ptr(), is_srgb as i32, priority)
			})?,
		})
	}
	pub fn create(
		sk: &StereoKit,
		texture_type: TextureType,
		format: TextureFormat,
	) -> Option<Self> {
		Some(Texture {
			sk: sk.get_wrapper(),
			tex: NonNull::new(unsafe {
				stereokit_sys::tex_create(transmute::<u32,IntType>(texture_type.bits() as u32), transmute::<u32,IntType>(format as u32))
			})?,
		})
	}
	pub fn from_mem(sk: &StereoKit, memory: &[u8], srgb_data: bool, priority: i32) -> Option<Self> {
		Some(Texture {
			sk: sk.get_wrapper(),
			tex: NonNull::new(unsafe {
				stereokit_sys::tex_create_mem(
					memory.as_ptr() as *mut c_void,
					memory.len() as u64,
					srgb_data as i32,
					priority,
				)
			})?,
		})
	}
	pub fn from_color32(
		sk: &StereoKit,
		data: Color32,
		width: i32,
		height: i32,
		uses_srgb_data: bool,
	) -> Option<Self> {
		Some(Texture {
			sk: sk.get_wrapper(),
			tex: NonNull::new(unsafe {
				stereokit_sys::tex_create_color32(
					&mut color32_from(data),
					width,
					height,
					uses_srgb_data as i32,
				)
			})?,
		})
	}
	pub fn from_cubemap_equirectangular(
		sk: &StereoKit,
		file_path: impl AsRef<Path>,
		uses_srgb_data: bool,
		load_priority: i32,
	) -> Option<(Self, SphericalHarmonics)> {
		let file_path = ustr(file_path.as_ref().as_os_str().to_str()?);
		let mut spherical_harmonics = SphericalHarmonics::default();
		let tex = NonNull::new(unsafe {
			stereokit_sys::tex_create_cubemap_file(
				file_path.as_char_ptr(),
				uses_srgb_data.into(),
				&mut spherical_harmonics.spherical_harmonics,
				load_priority,
			)
		})?;
		Some((
			Texture {
				sk: sk.get_wrapper(),
				tex,
			},
			spherical_harmonics,
		))
	}
	pub fn cubemap_from_gradient(
		sk: &StereoKit,
		gradient: &Gradient,
		direction: Vec3,
		resolution: u32,
		spherical_harmonics_info: Option<&mut SphericalHarmonics>,
	) -> Option<Self> {
		Some(Texture {
			sk: sk.get_wrapper(),
			tex: NonNull::new(unsafe {
				stereokit_sys::tex_gen_cubemap(
					std::mem::transmute(gradient),
					std::mem::transmute(direction),
					resolution as i32,
					spherical_harmonics_info
						.map(|info| (&mut info.spherical_harmonics) as *mut spherical_harmonics_t)
						.unwrap_or(std::ptr::null_mut()),
				)
			})?,
		})
	}
	pub fn cubemap_from_spherical_harmonics(
		sk: &StereoKit,
		spherical_harmonics: &SphericalHarmonics,
		resolution: u32,
		light_spot_size_pct: f32,
		light_spot_intensity: f32,
	) -> Option<Self> {
		Some(Texture {
			sk: sk.get_wrapper(),
			tex: NonNull::new(unsafe {
				stereokit_sys::tex_gen_cubemap_sh(
					&spherical_harmonics.spherical_harmonics,
					resolution as i32,
					light_spot_size_pct,
					light_spot_intensity,
				)
			})?,
		})
	}

	pub unsafe fn set_native(
		&self,
		native_texture: usize,
		native_format: i64,
		texture_type: TextureType,
		width: u32,
		height: u32,
		owned: bool,
	) {
		stereokit_sys::tex_set_surface(
			self.tex.as_ptr(),
			native_texture as *mut c_void,
			transmute::<u32,IntType>(texture_type.bits() as u32),
			native_format,
			width as i32,
			height as i32,
			1,
			owned as bool32_t,
		);
	}

	pub unsafe fn set_sample(&self, sample: TextureSample) {
		stereokit_sys::tex_set_sample(self.tex.as_ptr(), transmute::<u32,IntType>(sample as u32));
	}
	pub unsafe fn set_address_mode(&self, address_mode: TextureAddress) {
		stereokit_sys::tex_set_address(self.tex.as_ptr(), transmute::<u32,IntType>(address_mode as u32));
	}
	pub unsafe fn set_anisotropy_level(&self, anisotropy_level: i32) {
		stereokit_sys::tex_set_anisotropy(self.tex.as_ptr(), anisotropy_level);
	}
}
impl Drop for Texture {
	fn drop(&mut self) {
		unsafe { stereokit_sys::tex_release(self.tex.as_ptr()) }
	}
}
