use std::marker::PhantomData;
use stereokit_sys::{bool32_t, ui_cut_};
use crate::bounds::Bounds;
use crate::pose::Pose;
use crate::ui::WindowContext;
use crate::values::{MVec2, MVec3, pose_from, vec2_from, vec3_from, vec3_to};

#[derive(Copy, Clone, Debug)]
pub enum Side {
    Left = 0,
    Right = 1,
    Top = 2,
    Bottom = 3,
}
pub fn layout(_: &impl ValidLayout, start: MVec3, dimensions: MVec2, layout: impl FnOnce(&LayoutContext)) {
    _ui_layout_push(start, dimensions, false);
    layout(&LayoutContext::new());
    _ui_layout_pop();
}
pub fn layout_with_margin(_: &impl ValidLayout, start: MVec3, dimensions: MVec2, layout: impl FnOnce(&LayoutContext)) {
    _ui_layout_push(start, dimensions, true);
    layout(&LayoutContext::new());
    _ui_layout_pop();
}
pub fn layout_cut(_: &impl ValidLayout, cut_to: Side, size: f32, layout: impl FnOnce(&LayoutContext)) {
    _ui_layout_push_cut(cut_to, size, false);
    layout(&LayoutContext::new());
    _ui_layout_pop();
}
pub fn layout_cut_with_margin(_: &impl ValidLayout, cut_to: Side, size: f32, layout: impl FnOnce(&LayoutContext)) {
    _ui_layout_push_cut(cut_to, size, true);
    layout(&LayoutContext::new());
    _ui_layout_pop();
}
pub trait ValidLayout {}
pub struct LayoutContext(pub(crate) PhantomData<*const ()>);
impl ValidLayout for LayoutContext{}
impl ValidLayout for WindowContext{}
impl LayoutContext {
    fn new() -> Self {
        Self(Default::default())
    }
}
impl LayoutContext {
    pub fn ui(&self, ui: impl FnOnce(&WindowContext)) {
        ui(&WindowContext(PhantomData))
    }
    pub fn layout_at(&self) -> MVec3 {
      _ui_layout_at()
    }
    pub fn layout_last(&self) -> Bounds {
        _ui_layout_last()
    }
    pub fn layout_reserve(&self, size: MVec2, depth: f32) -> Bounds {
        _ui_layout_reserve(size, false, depth)
    }
    pub fn layout_reserve_with_padding(&self, size: MVec2, depth: f32) -> Bounds {
        _ui_layout_reserve(size, true, depth)
    }
}

pub fn _ui_layout_reserve(size: MVec2, add_padding: bool, depth: f32) -> Bounds {
    Bounds::from(unsafe {
        stereokit_sys::ui_layout_reserve(vec2_from(size), add_padding as bool32_t, depth)
    })
}
pub fn _ui_layout_last() -> Bounds {
    Bounds::from(unsafe {
        stereokit_sys::ui_layout_last()
    })
}
pub fn _ui_layout_at() -> MVec3 {
    vec3_to(unsafe {
        stereokit_sys::ui_layout_at()
    })
}
pub fn _ui_layout_push(start: MVec3, dimensions: MVec2, add_margin: bool) {
    unsafe {
        stereokit_sys::ui_layout_push(vec3_from(start), vec2_from(dimensions), add_margin as bool32_t)
    }
}
pub fn _ui_layout_push_cut(cut_to: Side, size: f32, add_margin: bool) {
    unsafe {
        stereokit_sys::ui_layout_push_cut(cut_to as u32 as ui_cut_, size, add_margin as bool32_t)
    }
}
pub fn _ui_layout_pop() {
    unsafe {
        stereokit_sys::ui_layout_pop();
    }
}