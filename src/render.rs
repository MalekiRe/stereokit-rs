#![allow(non_upper_case_globals)]

use std::fmt::Debug;

use crate::values::{IntegerType, matrix_from, matrix_to, MMatrix};
use crate::{texture::Texture, StereoKit};
use bitflags::bitflags;
use stereokit_sys::{_gradient_t, backend_xr_get_type, matrix, rect_t, vec3};
use crate::lifecycle::StereoKitContext;
use crate::render::BackendXrType::{OpenXr, Simulator, WebXr};
use crate::render::RenderClear::{Color, Depth};

bitflags! {
	pub struct RenderLayer: u32 {
		const Layer0 = 1 << 0;
		const Layer1 = 1 << 1;
		const Layer2 = 1 << 2;
		const Layer3 = 1 << 3;
		const Layer4 = 1 << 4;
		const Layer5 = 1 << 5;
		const Layer6 = 1 << 6;
		const Layer7 = 1 << 7;
		const Layer8 = 1 << 8;
		const Layer9 = 1 << 9;
		const LayerVFX = 10;
		const LayerAll = 0xFFFF;
		const LayerAllRegular = Self::Layer0.bits | Self::Layer1.bits | Self::Layer2.bits | Self::Layer3.bits | Self::Layer4.bits | Self::Layer5.bits | Self::Layer6.bits | Self::Layer7.bits | Self::Layer8.bits | Self::Layer9.bits;
	}
}

#[derive(Clone, Copy)]
pub struct SphericalHarmonics {
	pub(crate) spherical_harmonics: stereokit_sys::spherical_harmonics_t,
}
impl Default for SphericalHarmonics {
	fn default() -> Self {
		Self {
			spherical_harmonics: stereokit_sys::spherical_harmonics_t {
				coefficients: [vec3 {
					x: 0.0,
					y: 0.0,
					z: 0.0,
				}; 9],
			},
		}
	}
}
impl Debug for SphericalHarmonics {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("SphericalHarmonics")
			.field("coefficients", &self.spherical_harmonics.coefficients)
			.finish()
	}
}

#[derive(Debug, Copy, Clone)]
pub enum BackendXrType {
	None = 0,
	Simulator = 1,
	OpenXr = 2,
	WebXr = 3,
}

#[derive(Debug, Copy, Clone)]
pub enum RenderClear {
	None = 0,
	Color = 1,
	Depth = 2,
	All = 3,
}

#[derive(Debug, Copy, Clone)]
pub struct Rect {
	pub x: f32,
	pub y: f32,
	pub w: f32,
	pub h: f32,
}

impl From<rect_t> for Rect {
	fn from(value: rect_t) -> Self {
		Self {
			x: value.x,
			y: value.y,
			w: value.w,
			h: value.h,
		}
	}
}

impl Into<rect_t> for Rect {
	fn into(self) -> rect_t {
		rect_t {
			x: self.x,
			y: self.y,
			w: self.w,
			h: self.h,
		}
	}
}

impl From<IntegerType> for RenderClear {
	fn from(value: IntegerType) -> Self {
		match value {
			0 => RenderClear::None,
			1 => Color,
			2 => Depth,
			3 => RenderClear::All,
			_ => panic!()
		}
	}
}

impl Into<IntegerType> for RenderClear {
	fn into(self) -> IntegerType {
		self as IntegerType
	}
}

impl From<IntegerType> for BackendXrType {
	fn from(value: IntegerType) -> Self {
		match value {
			0 => BackendXrType::None,
			1 => Simulator,
			2 => OpenXr,
			3 => WebXr,
			_ => panic!()
		}
	}
}

pub trait StereoKitRender {
	fn set_skylight(&self, light: &SphericalHarmonics) {
		unsafe {
			stereokit_sys::render_set_skylight(&light.spherical_harmonics);
		}
	}

	fn set_skytex(&self, tex: &Texture) {
		unsafe {
			stereokit_sys::render_set_skytex(tex.tex.as_ptr());
		}
	}

	fn backend_xr_get_type(&self) -> BackendXrType {
		BackendXrType::from(unsafe {
			backend_xr_get_type()
		})
	}
	fn render_to(&self, to_rendertarget: &Texture, camera: impl Into<MMatrix>, projection: impl Into<MMatrix>, layer_filter: RenderLayer, clear: RenderClear, viewport: Rect) {
		unsafe {
			stereokit_sys::render_to(to_rendertarget.tex.as_ptr(), &camera.into().into(), &projection.into().into(), layer_filter.bits as stereokit_sys::render_layer_, clear.into(), viewport.into())
		}
	}
}
stereokit_trait_impl!(StereoKitRender);
impl StereoKit {}

pub struct Camera {}
impl Camera {
	pub fn set_root(_sk: &impl StereoKitContext, matrix: impl Into<MMatrix>) {
		let matrix = matrix.into();
		unsafe {
			stereokit_sys::render_set_cam_root(&matrix.into());
		}
	}
	pub fn get_root(_sk: &impl StereoKitContext) -> MMatrix {
		matrix_to(unsafe {
			stereokit_sys::render_get_cam_root()
		})
	}
}
