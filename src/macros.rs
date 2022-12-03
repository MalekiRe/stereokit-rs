macro_rules! stereokit_trait_impl {
	($t:ident)=> {
		impl $t for crate::lifecycle::StereoKit {}
		impl $t for crate::lifecycle::StereoKitDraw {}
	}
}