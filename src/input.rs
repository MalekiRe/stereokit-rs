#![allow(non_upper_case_globals)]

use crate::{
	pose::Pose,
	values::{MQuat, MVec2, MVec3},
	StereoKit,
};
use bitflags::bitflags;
use derive_is_enum_variant::is_enum_variant;
use num_derive::FromPrimitive;
use num_enum::TryFromPrimitive;
use std::ops::Deref;
use std::slice::Iter;
use std::{fmt::Pointer, mem::transmute};
use stereokit_sys::{bool32_t, button_state_, controller_t, hand_t, input_hand_visible, input_key, key_, quat, vec3};
use crate::values::{pose_to, quat_to, vec2_to, vec3_to};

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum Key {
	None = 0,
	MouseLeft = 1,
	MouseRight = 2,
	MouseCenter = 4,
	MouseForward = 5,
	MouseBack = 6,
	Backspace = 8,
	Tab = 9,
	Return = 13,
	Shift = 16,
	Ctrl = 17,
	Alt = 18,
	CapsLock = 20,
	Esc = 27,
	Space = 32,
	End = 35,
	Home = 36,
	Left = 37,
	Right = 39,
	Up = 38,
	Down = 40,
	PageUp = 33,
	PageDown = 34,
	Printscreen = 42,
	Insert = 45,
	Del = 46,
	Key0 = 48,
	Key1 = 49,
	Key2 = 50,
	Key3 = 51,
	Key4 = 52,
	Key5 = 53,
	Key6 = 54,
	Key7 = 55,
	Key8 = 56,
	Key9 = 57,
	KeyA = 65,
	KeyB = 66,
	KeyC = 67,
	KeyD = 68,
	KeyE = 69,
	KeyF = 70,
	KeyG = 71,
	KeyH = 72,
	KeyI = 73,
	KeyJ = 74,
	KeyK = 75,
	KeyL = 76,
	KeyM = 77,
	KeyN = 78,
	KeyO = 79,
	KeyP = 80,
	KeyQ = 81,
	KeyR = 82,
	KeyS = 83,
	KeyT = 84,
	KeyU = 85,
	KeyV = 86,
	KeyW = 87,
	KeyX = 88,
	KeyY = 89,
	KeyZ = 90,
	KeyNum0 = 96,
	KeyNum1 = 97,
	KeyNum2 = 98,
	KeyNum3 = 99,
	KeyNum4 = 100,
	KeyNum5 = 101,
	KeyNum6 = 102,
	KeyNum7 = 103,
	KeyNum8 = 104,
	KeyNum9 = 105,
	KeyF1 = 112,
	KeyF2 = 113,
	KeyF3 = 114,
	KeyF4 = 115,
	KeyF5 = 116,
	KeyF6 = 117,
	KeyF7 = 118,
	KeyF8 = 119,
	KeyF9 = 120,
	KeyF10 = 121,
	KeyF11 = 122,
	KeyF12 = 123,
	Comma = 188,
	Period = 190,
	SlashFwd = 191,
	SlashBack = 220,
	Semicolon = 186,
	Apostrophe = 222,
	BracketOpen = 219,
	BracketClose = 221,
	Minus = 189,
	Equals = 187,
	Backtick = 192,
	LCmd = 91,
	RCmd = 92,
	Multiply = 106,
	Add = 107,
	Subtract = 109,
	Decimal = 110,
	Divide = 111,
	MAX = 255,
}
bitflags! {
	pub struct ButtonState: u32 {
		const Inactive = 0;
		const Active = 1;
		const JustInactive = 2;
		const JustActive = 4;
		const Changed = 6;
	}
}

#[derive(Debug, Copy, Clone, is_enum_variant, TryFromPrimitive)]
#[repr(u32)]
pub enum TrackState {
	Lost = 0,
	Inferred = 1,
	Known = 2,
}

impl TrackState {
	pub fn try_from(val: u32) -> Option<Self> {
		Some(match val {
			0 => TrackState::Lost,
			1 => TrackState::Inferred,
			2 => TrackState::Known,
			_ => return None,
		})
	}
}

pub struct Ray {
	pub pos: MVec3,
	pub dir: MVec3,
}
impl Ray {
	pub fn from_mouse(mouse: &Mouse) -> Option<Self> {
		let mut ray = Ray {
			pos: MVec3::from([0.0, 0.0, 0.0]),
			dir: MVec3::from([0.0, 0.0, 0.0]),
		};

		unsafe { stereokit_sys::ray_from_mouse(transmute(mouse.pos), transmute(&mut ray)) != 0 }
			.then_some(ray)
	}
}

pub struct Mouse {
	available: i32,
	pub pos: MVec2,
	pub pos_change: MVec2,
	pub scroll: f32,
	pub scroll_change: f32,
}
impl Mouse {
	pub fn available(&self) -> bool {
		self.available != 0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u32)]
pub enum Handed {
	Left = 0,
	Right = 1,
}

impl Handed {
	pub(crate) fn from_sk(val: u32) -> Option<Self> {
		Some(match val {
			0 => Handed::Left,
			1 => Handed::Right,
			_ => return None
		})
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Joint {
	pub position: MVec3,
	pub orientation: MQuat,
	pub radius: f32,
}

impl Joint {
	pub(crate) fn from_sk_vals(pos: vec3, orientation: quat, radius: f32) -> Self {
		Self {
			position: vec3_to(pos),
			orientation: quat_to(orientation),
			radius,
		}
	}
}

/// The fingers go thumb to little, metacarpal to tip
#[derive(Debug, Clone, Copy)]
pub struct Hand {
	pub fingers: [[Joint; 5]; 5],
	pub wrist: Pose,
	pub palm: Pose,
	pub pinch_point: MVec3,
	pub handedness: Handed,
	pub tracked_state: ButtonState,
	pub pinch_state: ButtonState,
	pub grip_state: ButtonState,
	pub size: f32,
	pub pinch_activation: f32,
	pub grip_activation: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Controller {
	pub pose: Pose,
	pub palm: Pose,
	pub aim: Pose,
	pub tracked: ButtonState,
	pub tracked_pos: TrackState,
	pub tracked_rot: TrackState,
	pub stick_click: ButtonState,
	pub x1: ButtonState,
	pub x2: ButtonState,
	pub trigger: f32,
	pub grip: f32,
	pub stick: MVec2,
}

pub trait StereoKitInput {
	fn input_hand(&self, handed: Handed) -> Hand {
		let input_hand = unsafe { *stereokit_sys::input_hand((handed as u32).try_into().unwrap()) };
		match input_hand {
			hand_t { fingers, wrist, palm, pinch_pt, handedness, tracked_state, pinch_state, grip_state, size, pinch_activation, grip_activation } => {
				Hand {
					fingers: fingers.map(|t| {
						t.map(|a| Joint::from_sk_vals(a.position, a.orientation, a.radius))
					}),
					wrist: pose_to(wrist),
					palm: pose_to(palm),
					pinch_point: vec3_to(pinch_pt),
					handedness: Handed::from_sk(handedness.try_into().unwrap()).unwrap(),
					tracked_state: ButtonState::from_bits(tracked_state.try_into().unwrap()).unwrap(),
					pinch_state: ButtonState::from_bits(pinch_state.try_into().unwrap()).unwrap(),
					grip_state: ButtonState::from_bits(grip_state.try_into().unwrap()).unwrap(),
					size,
					pinch_activation,
					grip_activation,
				}
			}
		}
	}
	fn input_controller(&self, handed: Handed) -> Controller {
		let controller = unsafe { *stereokit_sys::input_controller((handed as u32).try_into().unwrap()) };
		match controller {
			controller_t { pose, palm, aim, tracked, tracked_pos, tracked_rot, stick_click, x1, x2, trigger, grip, stick } => {
				Controller {
					pose: pose_to(pose),
					palm: pose_to(palm),
					aim: pose_to(aim),
					tracked: ButtonState::from_bits(tracked.try_into().unwrap()).unwrap(),
					tracked_pos: TrackState::try_from(tracked_pos.try_into().unwrap()).unwrap(),
					tracked_rot: TrackState::try_from(tracked_rot.try_into().unwrap()).unwrap(),
					stick_click: ButtonState::from_bits(stick_click.try_into().unwrap()).unwrap(),
					x1: ButtonState::from_bits(x1.try_into().unwrap()).unwrap(),
					x2: ButtonState::from_bits(x2.try_into().unwrap()).unwrap(),
					trigger,
					grip,
					stick: vec2_to(stick),
				}
			}
		}
	}
	fn input_controller_menu(&self) -> ButtonState {
		let button_state = unsafe { stereokit_sys::input_controller_menu() };
		ButtonState::from_bits(button_state.try_into().unwrap()).unwrap()
	}
	fn input_hand_visible(&self, handed: Handed, visible: bool) {
		unsafe { input_hand_visible((handed as u32).try_into().unwrap(), visible as bool32_t) }
	}
	fn input_head(&self) -> Pose {
		let pose = unsafe {*stereokit_sys::input_head()};
		Pose {
			position: vec3_to(pose.position),
			orientation: quat_to(pose.orientation),
		}
	}

	fn input_mouse(&self) -> &Mouse {
		unsafe { transmute(&*stereokit_sys::input_mouse()) }
	}

	fn input_key(&self, key: Key) -> ButtonState {
		ButtonState::from_bits_truncate(unsafe { input_key(key as key_).try_into().unwrap() })
	}
}
stereokit_trait_impl!(StereoKitInput);
