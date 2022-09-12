#![allow(non_upper_case_globals)]

use crate::{texture::Texture, StereoKit};
use bitflags::bitflags;

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

pub struct SphericalHarmonics {
	pub(crate) spherical_harmonics: stereokit_sys::spherical_harmonics_t,
}

pub fn set_skylight(sk: &StereoKit, light: &SphericalHarmonics) {
	unsafe {
		stereokit_sys::render_set_skylight(&light.spherical_harmonics);
	}
}

pub fn set_skytex(sk: &StereoKit, tex: &Texture) {
	unsafe {
		stereokit_sys::render_set_skytex(tex.tex.as_ptr());
	}
}
