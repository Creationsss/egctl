use std::fmt;

use anyhow::{bail, Result};

use crate::protocol::*;
use crate::types::mapping::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollingRate {
	Hz1000,
	Hz2000,
	Hz4000,
	Hz8000,
}

impl PollingRate {
	pub fn from_hz(hz: u32) -> Result<Self> {
		match hz {
			1000 => Ok(Self::Hz1000),
			2000 => Ok(Self::Hz2000),
			4000 => Ok(Self::Hz4000),
			8000 => Ok(Self::Hz8000),
			_ => bail!("invalid polling rate {hz}. must be one of: 1000, 2000, 4000, 8000"),
		}
	}

	pub fn to_hz(self) -> u32 {
		match self {
			Self::Hz1000 => 1000,
			Self::Hz2000 => 2000,
			Self::Hz4000 => 4000,
			Self::Hz8000 => 8000,
		}
	}

	pub fn to_divider(self) -> u8 {
		match self {
			Self::Hz8000 => 1,
			Self::Hz4000 => 2,
			Self::Hz2000 => 4,
			Self::Hz1000 => 8,
		}
	}

	pub fn from_divider(d: u8) -> Self {
		match d {
			0 | 1 => Self::Hz8000,
			2 => Self::Hz4000,
			3 | 4 => Self::Hz2000,
			_ => Self::Hz1000,
		}
	}
}

impl fmt::Display for PollingRate {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} Hz", self.to_hz())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpdtMode {
	Off,
	Safe,
	Speed,
}

impl SpdtMode {
	pub fn from_byte(b: u8) -> (Self, u8) {
		if b >= SPDT_THRESHOLD {
			let mode = match b {
				SPDT_SAFE => Self::Safe,
				SPDT_SPEED => Self::Speed,
				_ => Self::Off,
			};
			(mode, 0)
		} else {
			(Self::Off, b)
		}
	}

	pub fn to_byte(self, multiclick: u8) -> u8 {
		match self {
			Self::Off => multiclick.min(25),
			Self::Safe => SPDT_SAFE,
			Self::Speed => SPDT_SPEED,
		}
	}
}

impl fmt::Display for SpdtMode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Off => write!(f, "off"),
			Self::Safe => write!(f, "safe"),
			Self::Speed => write!(f, "speed"),
		}
	}
}

#[derive(Debug, Clone)]
pub struct CpiLevel {
	pub xy_split: bool,
	pub x: u16,
	pub y: u16,
}

#[derive(Debug, Clone)]
pub struct ButtonConfig {
	pub spdt: SpdtMode,
	pub multiclick: u8,
	pub mapping: MappingType,
}

#[derive(Debug, Clone)]
pub struct MouseConfig {
	pub raw: [u8; CONFIG_SIZE],
	pub polling_rate: PollingRate,
	pub slamclick_filter: bool,
	pub jitter_filter: bool,
	pub lod: u8,
	pub angle_snapping: bool,
	pub ripple_control: bool,
	pub motion_sync: bool,
	pub cpi_levels: u8,
	pub cpis: [CpiLevel; CPI_COUNT],
	pub buttons: [ButtonConfig; BUTTON_COUNT],
}

const BUTTON_NAMES: [&str; BUTTON_COUNT] = [
	"Left", "Right", "Middle", "Forward", "Back", "Button 6", "Button 7",
];

impl MouseConfig {
	pub fn from_bytes(buf: &[u8; CONFIG_SIZE]) -> Self {
		let divider = buf[OFF_POLLING_DIVIDER];
		let flags = buf[OFF_FILTER_FLAGS];

		let cpis: [CpiLevel; CPI_COUNT] = std::array::from_fn(|i| {
			let base = OFF_CPIS + i * CPI_STRUCT_SIZE;
			CpiLevel {
				xy_split: buf[base] != 0,
				x: u16::from_le_bytes([buf[base + 1], buf[base + 2]]),
				y: u16::from_le_bytes([buf[base + 3], buf[base + 4]]),
			}
		});

		let buttons: [ButtonConfig; BUTTON_COUNT] = std::array::from_fn(|i| {
			let base = OFF_BUTTONS + i * BUTTON_STRUCT_SIZE;
			let (spdt, multiclick) = SpdtMode::from_byte(buf[base]);
			let mapping = MappingType::from_bytes(buf[base + 1] as i8, buf[base + 2]);
			ButtonConfig {
				spdt,
				multiclick,
				mapping,
			}
		});

		Self {
			raw: *buf,
			polling_rate: PollingRate::from_divider(divider),
			slamclick_filter: flags & FILTER_SLAMCLICK != 0,
			jitter_filter: flags & FILTER_JITTER != 0,
			lod: buf[OFF_LOD],
			angle_snapping: buf[OFF_ANGLE_SNAPPING] != 0,
			ripple_control: buf[OFF_RIPPLE_CONTROL] != 0,
			motion_sync: buf[OFF_MOTION_SYNC] != 0,
			cpi_levels: buf[OFF_CPI_LEVELS],
			cpis,
			buttons,
		}
	}

	pub fn to_bytes(&self) -> [u8; CONFIG_SIZE] {
		let mut buf = self.raw;

		buf[OFF_POLLING_DIVIDER] = self.polling_rate.to_divider();

		let mut flags = buf[OFF_FILTER_FLAGS] & !(FILTER_SLAMCLICK | FILTER_JITTER);
		if self.slamclick_filter {
			flags |= FILTER_SLAMCLICK;
		}
		if self.jitter_filter {
			flags |= FILTER_JITTER;
		}
		buf[OFF_FILTER_FLAGS] = flags;

		buf[OFF_LOD] = self.lod;
		buf[OFF_ANGLE_SNAPPING] = self.angle_snapping as u8;
		buf[OFF_RIPPLE_CONTROL] = self.ripple_control as u8;
		buf[OFF_MOTION_SYNC] = self.motion_sync as u8;
		buf[OFF_CPI_LEVELS] = self.cpi_levels;

		for i in 0..CPI_COUNT {
			let base = OFF_CPIS + i * CPI_STRUCT_SIZE;
			buf[base] = self.cpis[i].xy_split as u8;
			let x = self.cpis[i].x.to_le_bytes();
			let y = self.cpis[i].y.to_le_bytes();
			buf[base + 1] = x[0];
			buf[base + 2] = x[1];
			buf[base + 3] = y[0];
			buf[base + 4] = y[1];
		}

		for i in 0..BUTTON_COUNT {
			let base = OFF_BUTTONS + i * BUTTON_STRUCT_SIZE;
			buf[base] = self.buttons[i].spdt.to_byte(self.buttons[i].multiclick);
			let (type_byte, value_byte) = self.buttons[i].mapping.to_bytes();
			buf[base + 1] = type_byte as u8;
			buf[base + 2] = value_byte;
		}

		buf
	}
}

impl fmt::Display for MouseConfig {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "Polling rate:      {}", self.polling_rate)?;
		writeln!(f, "Slamclick filter:  {}", on_off(self.slamclick_filter))?;
		writeln!(f, "Jitter filter:     {}", on_off(self.jitter_filter))?;
		writeln!(f, "Lift-off distance: {}", self.lod)?;
		writeln!(f, "Angle snapping:    {}", on_off(self.angle_snapping))?;
		writeln!(f, "Ripple control:    {}", on_off(self.ripple_control))?;
		writeln!(f, "Motion sync:       {}", on_off(self.motion_sync))?;
		writeln!(f, "Active CPI levels: {}", self.cpi_levels)?;
		for (i, cpi) in self.cpis.iter().enumerate() {
			let active = if (i as u8) < self.cpi_levels {
				""
			} else {
				" (inactive)"
			};
			if cpi.xy_split {
				writeln!(f, "  CPI {}: X={} Y={}{active}", i + 1, cpi.x, cpi.y)?;
			} else {
				writeln!(f, "  CPI {}: {}{active}", i + 1, cpi.x)?;
			}
		}
		writeln!(f, "Buttons:")?;
		for (i, btn) in self.buttons.iter().enumerate() {
			let name = BUTTON_NAMES.get(i).unwrap_or(&"Unknown");
			write!(f, "  {name}: {}", btn.mapping)?;
			if btn.spdt != SpdtMode::Off {
				write!(f, " [SPDT: {}]", btn.spdt)?;
			} else if btn.multiclick > 0 {
				write!(f, " [debounce: {}]", btn.multiclick)?;
			}
			writeln!(f)?;
		}
		Ok(())
	}
}

pub fn validate_dpi(dpi: u16) -> Result<u16> {
	if !(CPI_MIN..=CPI_MAX).contains(&dpi) {
		bail!("invalid DPI value {dpi}. must be {CPI_MIN}-{CPI_MAX} in steps of {CPI_STEP}");
	}
	Ok(((dpi as u32 + CPI_STEP as u32 / 2) / CPI_STEP as u32 * CPI_STEP as u32) as u16)
}

pub fn on_off(b: bool) -> &'static str {
	if b {
		"on"
	} else {
		"off"
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn make_test_buf() -> [u8; CONFIG_SIZE] {
		let mut buf = [0u8; CONFIG_SIZE];
		buf[0] = 0xa1;
		buf[1] = 0x00;
		buf[OFF_POLLING_DIVIDER] = 1;
		buf[OFF_FILTER_FLAGS] = FILTER_SLAMCLICK;
		buf[OFF_LOD] = 1;
		buf[OFF_ANGLE_SNAPPING] = 0;
		buf[OFF_RIPPLE_CONTROL] = 1;
		buf[OFF_MOTION_SYNC] = 0;
		buf[OFF_CPI_LEVELS] = 2;
		buf[OFF_CPIS] = 0;
		buf[OFF_CPIS + 1] = 0x20;
		buf[OFF_CPIS + 2] = 0x03;
		buf[OFF_CPIS + 3] = 0x20;
		buf[OFF_CPIS + 4] = 0x03;
		buf[OFF_CPIS + 5] = 1;
		buf[OFF_CPIS + 6] = 0x90;
		buf[OFF_CPIS + 7] = 0x01;
		buf[OFF_CPIS + 8] = 0x40;
		buf[OFF_CPIS + 9] = 0x06;
		buf[OFF_BUTTONS] = 8;
		buf[OFF_BUTTONS + 1] = MAP_MOUSE as u8;
		buf[OFF_BUTTONS + 2] = MOUSE_LEFT;
		buf[OFF_BUTTONS + 3] = SPDT_SPEED;
		buf[OFF_BUTTONS + 4] = MAP_MOUSE as u8;
		buf[OFF_BUTTONS + 5] = MOUSE_RIGHT;
		buf[OFF_BUTTONS + 6] = 0;
		buf[OFF_BUTTONS + 7] = MAP_MOUSE as u8;
		buf[OFF_BUTTONS + 8] = MOUSE_MIDDLE;
		buf[OFF_BUTTONS + 9] = 0;
		buf[OFF_BUTTONS + 10] = MAP_MOUSE as u8;
		buf[OFF_BUTTONS + 11] = MOUSE_FORWARD;
		buf[OFF_BUTTONS + 12] = 0;
		buf[OFF_BUTTONS + 13] = MAP_MOUSE as u8;
		buf[OFF_BUTTONS + 14] = MOUSE_BACK;
		buf[OFF_BUTTONS + 15] = 0;
		buf[OFF_BUTTONS + 16] = MAP_CPI_LOOP as u8;
		buf[OFF_BUTTONS + 17] = 0;
		buf[OFF_BUTTONS + 18] = 0;
		buf[OFF_BUTTONS + 19] = MAP_DISABLE as u8;
		buf[OFF_BUTTONS + 20] = 0;
		buf
	}

	#[test]
	fn round_trip() {
		let buf = make_test_buf();
		let config = MouseConfig::from_bytes(&buf);
		let out = config.to_bytes();
		for i in 0..(CPI_COUNT * CPI_STRUCT_SIZE) {
			assert_eq!(
				out[OFF_CPIS + i],
				buf[OFF_CPIS + i],
				"CPI byte {i} mismatch"
			);
		}
		for i in 0..(BUTTON_COUNT * BUTTON_STRUCT_SIZE) {
			assert_eq!(
				out[OFF_BUTTONS + i],
				buf[OFF_BUTTONS + i],
				"button byte {i} mismatch"
			);
		}
		assert_eq!(out[OFF_POLLING_DIVIDER], buf[OFF_POLLING_DIVIDER]);
		assert_eq!(out[OFF_FILTER_FLAGS], buf[OFF_FILTER_FLAGS]);
		assert_eq!(out[OFF_LOD], buf[OFF_LOD]);
	}

	#[test]
	fn parse_fields() {
		let buf = make_test_buf();
		let config = MouseConfig::from_bytes(&buf);

		assert_eq!(config.polling_rate, PollingRate::Hz8000);
		assert!(config.slamclick_filter);
		assert!(!config.jitter_filter);
		assert_eq!(config.lod, 1);
		assert!(!config.angle_snapping);
		assert!(config.ripple_control);
		assert!(!config.motion_sync);
		assert_eq!(config.cpi_levels, 2);

		assert!(!config.cpis[0].xy_split);
		assert_eq!(config.cpis[0].x, 800);
		assert!(config.cpis[1].xy_split);
		assert_eq!(config.cpis[1].x, 400);
		assert_eq!(config.cpis[1].y, 1600);

		assert_eq!(config.buttons[0].spdt, SpdtMode::Off);
		assert_eq!(config.buttons[0].multiclick, 8);
		assert_eq!(
			config.buttons[0].mapping,
			MappingType::Mouse(MouseKey::Left)
		);

		assert_eq!(config.buttons[1].spdt, SpdtMode::Speed);
		assert_eq!(
			config.buttons[1].mapping,
			MappingType::Mouse(MouseKey::Right)
		);
	}

	#[test]
	fn modify_preserves_padding() {
		let mut buf = make_test_buf();
		buf[5] = 0xAB;
		buf[35] = 0xCD;
		buf[100] = 0xEF;

		let mut config = MouseConfig::from_bytes(&buf);
		config.polling_rate = PollingRate::Hz4000;
		config.lod = 2;

		let out = config.to_bytes();
		assert_eq!(out[5], 0xAB);
		assert_eq!(out[35], 0xCD);
		assert_eq!(out[100], 0xEF);
		assert_eq!(out[OFF_POLLING_DIVIDER], 2);
		assert_eq!(out[OFF_LOD], 2);
	}

	#[test]
	fn polling_rate_divider_conversions() {
		assert_eq!(PollingRate::Hz8000.to_divider(), 1);
		assert_eq!(PollingRate::Hz4000.to_divider(), 2);
		assert_eq!(PollingRate::Hz2000.to_divider(), 4);
		assert_eq!(PollingRate::Hz1000.to_divider(), 8);
		assert_eq!(PollingRate::from_divider(0), PollingRate::Hz8000);
		assert_eq!(PollingRate::from_divider(1), PollingRate::Hz8000);
		assert_eq!(PollingRate::from_divider(2), PollingRate::Hz4000);
		assert_eq!(PollingRate::from_divider(4), PollingRate::Hz2000);
		assert_eq!(PollingRate::from_divider(8), PollingRate::Hz1000);
	}

	#[test]
	fn validate_dpi_rounding() {
		assert_eq!(validate_dpi(800).unwrap(), 800);
		assert_eq!(validate_dpi(50).unwrap(), 50);
		assert_eq!(validate_dpi(26000).unwrap(), 26000);
		assert_eq!(validate_dpi(123).unwrap(), 100);
		assert_eq!(validate_dpi(126).unwrap(), 150);
		assert!(validate_dpi(0).is_err());
		assert!(validate_dpi(30000).is_err());
	}

	#[test]
	fn spdt_mode_encoding() {
		assert_eq!(SpdtMode::from_byte(0), (SpdtMode::Off, 0));
		assert_eq!(SpdtMode::from_byte(8), (SpdtMode::Off, 8));
		assert_eq!(SpdtMode::from_byte(SPDT_SAFE), (SpdtMode::Safe, 0));
		assert_eq!(SpdtMode::from_byte(SPDT_SPEED), (SpdtMode::Speed, 0));
		assert_eq!(SpdtMode::Off.to_byte(8), 8);
		assert_eq!(SpdtMode::Safe.to_byte(8), SPDT_SAFE);
		assert_eq!(SpdtMode::Speed.to_byte(8), SPDT_SPEED);
	}
}
