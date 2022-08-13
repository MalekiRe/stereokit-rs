use std::rc::{Rc, Weak};
use stereokit_sys::shader_t;

use crate::{lifecycle::StereoKitInstance, StereoKit};
pub struct Shader {
	pub(crate) sk: Weak<StereoKitInstance>,
	pub(crate) shader: shader_t,
}
