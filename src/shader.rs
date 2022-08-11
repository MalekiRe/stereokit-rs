use stereokit_sys::shader_t;

use crate::StereoKit;
pub struct Shader<'a> {
	pub(crate) sk: &'a StereoKit<'a>,
	pub(crate) shader: shader_t,
}
