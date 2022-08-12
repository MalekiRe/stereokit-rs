#![allow(non_upper_case_globals)]

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
