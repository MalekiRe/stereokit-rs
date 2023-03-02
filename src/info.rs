use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use stereokit_sys::sk_system_info;

use crate::StereoKit;
use crate::values::IntegerType;

// pub const display__display_none: display_ = 0;
// pub const display__display_opaque: display_ = 1;
// pub const display__display_additive: display_ = 2;
// pub const display__display_blend: display_ = 4;
// pub const display__display_passthrough: display_ = 4;
// pub const display__display_any_transparent: display_ = 6;

#[repr(u32)]
#[derive(Debug, Clone, Copy, TryFromPrimitive, Deserialize_repr, Serialize_repr)]
pub enum Display {
	None = 0,
	Opaque = 1,
	Additive = 2,
	Blend = 4,
	AnyTransparent = 6,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct SystemInfo {
	pub display_type: Display,
	pub display_width: u32,
	pub display_height: u32,
	pub spatial_bridge_present: bool,
	pub perception_bridge_present: bool,
	pub eye_tracking_present: bool,
	pub overlay_app: bool,
	pub world_occlusion_present: bool,
	pub world_raycast_present: bool,
}

impl StereoKit {
	pub fn system_info(&self) -> SystemInfo {
		let info = unsafe { sk_system_info() };
		SystemInfo {
			display_type: Display::try_from_primitive(info.display_type as u32).unwrap(),
			display_width: info.display_width as u32,
			display_height: info.display_height as u32,
			spatial_bridge_present: info.spatial_bridge_present > 0,
			perception_bridge_present: info.perception_bridge_present > 0,
			eye_tracking_present: info.eye_tracking_present > 0,
			overlay_app: info.overlay_app > 0,
			world_occlusion_present: info.world_occlusion_present > 0,
			world_raycast_present: info.world_raycast_present > 0,
		}
	}
}
