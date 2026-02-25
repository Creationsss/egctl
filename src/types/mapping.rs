use std::fmt;

use crate::protocol::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseKey {
	Left,
	Right,
	Middle,
	Back,
	Forward,
}

impl MouseKey {
	pub fn from_byte(b: u8) -> Option<Self> {
		match b {
			MOUSE_LEFT => Some(Self::Left),
			MOUSE_RIGHT => Some(Self::Right),
			MOUSE_MIDDLE => Some(Self::Middle),
			MOUSE_BACK => Some(Self::Back),
			MOUSE_FORWARD => Some(Self::Forward),
			_ => None,
		}
	}

	pub fn to_byte(self) -> u8 {
		match self {
			Self::Left => MOUSE_LEFT,
			Self::Right => MOUSE_RIGHT,
			Self::Middle => MOUSE_MIDDLE,
			Self::Back => MOUSE_BACK,
			Self::Forward => MOUSE_FORWARD,
		}
	}

	pub fn name(self) -> &'static str {
		match self {
			Self::Left => "left",
			Self::Right => "right",
			Self::Middle => "middle",
			Self::Back => "back",
			Self::Forward => "forward",
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
	Up,
	Down,
}

impl ScrollDirection {
	pub fn from_byte(b: u8) -> Self {
		if b as i8 > 0 {
			Self::Up
		} else {
			Self::Down
		}
	}

	pub fn to_byte(self) -> u8 {
		match self {
			Self::Up => 1u8,
			Self::Down => (-1i8) as u8,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKey {
	PlayPause,
	Next,
	Previous,
	Mute,
	VolumeUp,
	VolumeDown,
	Browser,
	Explorer,
}

impl MediaKey {
	pub fn from_byte(b: u8) -> Option<Self> {
		match b {
			MEDIA_PLAY_PAUSE => Some(Self::PlayPause),
			MEDIA_NEXT => Some(Self::Next),
			MEDIA_PREVIOUS => Some(Self::Previous),
			MEDIA_MUTE => Some(Self::Mute),
			MEDIA_VOLUME_UP => Some(Self::VolumeUp),
			MEDIA_VOLUME_DOWN => Some(Self::VolumeDown),
			MEDIA_BROWSER => Some(Self::Browser),
			MEDIA_EXPLORER => Some(Self::Explorer),
			_ => None,
		}
	}

	pub fn to_byte(self) -> u8 {
		match self {
			Self::PlayPause => MEDIA_PLAY_PAUSE,
			Self::Next => MEDIA_NEXT,
			Self::Previous => MEDIA_PREVIOUS,
			Self::Mute => MEDIA_MUTE,
			Self::VolumeUp => MEDIA_VOLUME_UP,
			Self::VolumeDown => MEDIA_VOLUME_DOWN,
			Self::Browser => MEDIA_BROWSER,
			Self::Explorer => MEDIA_EXPLORER,
		}
	}

	pub fn name(self) -> &'static str {
		match self {
			Self::PlayPause => "play-pause",
			Self::Next => "next",
			Self::Previous => "previous",
			Self::Mute => "mute",
			Self::VolumeUp => "volume-up",
			Self::VolumeDown => "volume-down",
			Self::Browser => "browser",
			Self::Explorer => "explorer",
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingType {
	Mouse(MouseKey),
	Scroll(ScrollDirection),
	Keyboard(u8),
	CpiLoop,
	Cpi(u8),
	Media(MediaKey),
	Disable,
}

impl MappingType {
	pub fn from_bytes(type_byte: i8, value_byte: u8) -> Self {
		match type_byte {
			MAP_MOUSE => {
				let key = MouseKey::from_byte(value_byte).unwrap_or(MouseKey::Left);
				Self::Mouse(key)
			}
			MAP_SCROLL => Self::Scroll(ScrollDirection::from_byte(value_byte)),
			MAP_KEYBOARD => Self::Keyboard(value_byte),
			MAP_CPI_LOOP => Self::CpiLoop,
			MAP_CPI => Self::Cpi(value_byte),
			MAP_MEDIA => {
				let key = MediaKey::from_byte(value_byte).unwrap_or(MediaKey::PlayPause);
				Self::Media(key)
			}
			_ => Self::Disable,
		}
	}

	pub fn to_bytes(self) -> (i8, u8) {
		match self {
			Self::Mouse(k) => (MAP_MOUSE, k.to_byte()),
			Self::Scroll(d) => (MAP_SCROLL, d.to_byte()),
			Self::Keyboard(code) => (MAP_KEYBOARD, code),
			Self::CpiLoop => (MAP_CPI_LOOP, 0),
			Self::Cpi(level) => (MAP_CPI, level),
			Self::Media(k) => (MAP_MEDIA, k.to_byte()),
			Self::Disable => (MAP_DISABLE, 0),
		}
	}
}

impl fmt::Display for MappingType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Mouse(k) => write!(f, "mouse {}", k.name()),
			Self::Scroll(ScrollDirection::Up) => write!(f, "scroll up"),
			Self::Scroll(ScrollDirection::Down) => write!(f, "scroll down"),
			Self::Keyboard(code) => write!(f, "key 0x{code:02X}"),
			Self::CpiLoop => write!(f, "CPI cycle"),
			Self::Cpi(level) => write!(f, "CPI level {}", level + 1),
			Self::Media(k) => write!(f, "media {}", k.name()),
			Self::Disable => write!(f, "disabled"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn mapping_type_round_trip() {
		let cases = [
			MappingType::Mouse(MouseKey::Left),
			MappingType::Mouse(MouseKey::Forward),
			MappingType::Scroll(ScrollDirection::Up),
			MappingType::Scroll(ScrollDirection::Down),
			MappingType::Keyboard(0x04),
			MappingType::CpiLoop,
			MappingType::Cpi(2),
			MappingType::Media(MediaKey::PlayPause),
			MappingType::Media(MediaKey::VolumeDown),
			MappingType::Disable,
		];
		for mapping in cases {
			let (t, v) = mapping.to_bytes();
			let decoded = MappingType::from_bytes(t, v);
			assert_eq!(decoded, mapping, "round-trip failed for {mapping:?}");
		}
	}
}
