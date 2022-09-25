#![allow(non_upper_case_globals)]

use std::fmt::Debug;

use crate::{texture::Texture, StereoKit};
use bitflags::bitflags;
use stereokit_sys::{_gradient_t, vec3};

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

impl StereoKit {
	pub fn set_skylight(&mut self, light: &SphericalHarmonics) {
		unsafe {
			stereokit_sys::render_set_skylight(&light.spherical_harmonics);
		}
	}

	pub fn set_skytex(&mut self, tex: &Texture) {
		unsafe {
			stereokit_sys::render_set_skytex(tex.tex.as_ptr());
		}
	}
}
