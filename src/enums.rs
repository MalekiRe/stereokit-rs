pub enum DisplayMode {
	MixedReality = 0,
	Flatscreen = 1,
	None = 2
}
pub enum DisplayBlend {
	None = 0,
	Opaque = 1,
	Additive = 2,
	Blend = 4,
	AnyTransparent = 6
}
pub enum DepthMode {
	Balanced = 0,
	d16 = 1,
	d32 = 2,
	Stencil = 3
}
pub enum LogFilter {
	None = 0,
	Diagnostic = 1,
	Inform = 2,
	Warning = 3,
	Error = 4
}
pub enum RenderLayer {
	Layer0 = 1,
	Layer1 = 2,
	Layer2 = 4,
	Layer3 = 8,
	Layer4 = 16,
	Layer5 = 32,
	Layer6 = 64,
	Layer7 = 128,
	Layer8 = 256,
	Layer9 = 512,
	LayerVFX = 1024,
	LayerAll = 65535,
	LayerAllRegular = 1023
}