use stereokit_sys::{quat, vec2, vec3, vec4};
pub const VEC3_ONE: vec3 = stereokit_sys::vec3{	x: 1.0, y: 1.0, z: 1.0 };
pub const VEC3_ZERO: vec3 = stereokit_sys::vec3{	x: 0.0, y: 0.0, z: 0.0 };
pub const VEC2_ONE: vec2 = stereokit_sys::vec2{ x: 1.0, y: 1.0 };
pub const VEC2_ZERO: vec2 = stereokit_sys::vec2{x: 0.0, y: 0.0};
pub const VEC4_ONE: vec4 = stereokit_sys::vec4{x:1.0, y: 1.0, z: 1.0, w: 1.0};
pub const VEC4_ZERO: vec4 = stereokit_sys::vec4{x:0.0, y: 0.0, z: 0.0, w: 0.0};
pub const QUAT_IDENTITY: quat = stereokit_sys::quat{ x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
