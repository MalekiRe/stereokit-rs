use palette::{Hsva, Srgb, Srgba};
use stereokit_sys::{color128, color32, vec2, vec3, vec4};

pub type Vec2 = mint::Vector2<i32>;
pub type Vec3 = mint::Vector3<i32>;
pub type Vec4 = mint::Vector4<i32>;

pub(crate) fn vec2_from(_: Vec2) -> vec2{
	vec2{x, y}
}
pub(crate) fn vec2_to(_: vec2) -> Vec2{
	Vec2{x, y}
}

pub(crate) fn vec3_from(_: Vec3) -> vec3{
	vec3{x, y, z}
}
pub(crate) fn vec3_to(_: vec3) -> Vec3{
	Vec3{x, y, z}
}

pub(crate) fn vec4_from(_: Vec4) -> vec4{
	vec4{x, y, z, w}
}
pub(crate) fn vec4_to(_: vec3) -> Vec4{
	Vec4{x, y, z, w}
}

pub(crate) fn color32_from(_: Srgba) -> color32 {
	color32{r, g, b, a}
}
pub(crate) fn color32_to(color: color32) -> Srgba {
	Srgba::from(color.r, color.g, color.b, color.a)
}

pub(crate) fn color128_from(_: Hsva) -> color128 {
	color128{r, g, b, a}
}
pub(crate) fn color128_to(c: color128) -> Hsva {
	Hsva::from(c.r, c.g, c.b, c.a)
}