#![allow(non_upper_case_globals)]

use bitflags::bitflags;

pub enum DisplayMode {
	MixedReality = 0,
	Flatscreen = 1,
	None = 2,
}
pub enum DisplayBlend {
	None = 0,
	Opaque = 1,
	Additive = 2,
	Blend = 4,
	AnyTransparent = 6,
}
pub enum DepthMode {
	Balanced = 0,
	D16 = 1,
	D32 = 2,
	Stencil = 3,
}
pub enum LogFilter {
	None = 0,
	Diagnostic = 1,
	Inform = 2,
	Warning = 3,
	Error = 4,
}

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

	pub struct TextAlign: u32 {
		const XLeft        = 1 << 0;
		const YTop         = 1 << 1;
		const XCenter      = 1 << 2;
		const YCenter      = 1 << 3;
		const XRight       = 1 << 4;
		const YBottom      = 1 << 5;
		const Center       = Self::XCenter.bits | Self::YCenter.bits;
		const CenterLeft   = Self::XLeft.bits   | Self::YCenter.bits;
		const CenterRight  = Self::XRight.bits  | Self::YCenter.bits;
		const TopCenter    = Self::XCenter.bits | Self::YTop.bits;
		const TopLeft      = Self::XLeft.bits   | Self::YTop.bits;
		const TopRight     = Self::XRight.bits  | Self::YTop.bits;
		const BottomCenter = Self::XCenter.bits | Self::YBottom.bits;
		const BottomLeft   = Self::XLeft.bits   | Self::YBottom.bits;
		const BottomRight  = Self::XRight.bits  | Self::YBottom.bits;
	}

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
