use std::rc::{Rc, Weak};
use stereokit_sys::shader_t;

use crate::{lifecycle::StereoKitInstanceWrapper, StereoKit};
pub struct Shader {
	pub(crate) sk: StereoKitInstanceWrapper,
	pub(crate) shader: shader_t,
}
