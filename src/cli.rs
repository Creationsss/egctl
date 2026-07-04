use std::str::FromStr;

use clap::{Parser, Subcommand, ValueEnum};

use crate::protocol::BUTTON_COUNT;
use crate::types::{key_code, modifier_bit};

#[derive(Parser)]
#[command(
	name = "egctl",
	about = "Configure the Endgame Gear OP1 8k v2 mouse on Linux",
	arg_required_else_help = true,
	after_help = "Examples:\n  \
		egctl info                       show current settings\n  \
		egctl dpi 1600                   set DPI for level 1\n  \
		egctl dpi 3200 --level 2         set DPI for level 2\n  \
		egctl rate 8000                  set polling rate to 8000 Hz\n  \
		egctl bind forward key f5        bind the forward button to F5\n  \
		egctl bind 6 media mute          bind button 6 to mute\n  \
		egctl keys                       list key names for 'bind ... key'"
)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
}

const BUTTON_HELP: &str = "Button: left, right, middle, forward, back, or a number 1-7";

#[derive(Subcommand)]
pub enum Commands {
	#[command(about = "Show firmware version and all current settings")]
	Info,

	#[command(about = "Show or set DPI for a CPI level")]
	Dpi {
		#[arg(
			value_name = "DPI",
			help = "DPI value (50-26000, steps of 50); omit to show the current value"
		)]
		value: Option<u16>,
		#[arg(short, long, default_value_t = 1, help = "CPI level (1-4)")]
		level: u8,
		#[arg(short = 'x', long, help = "X-axis DPI (for split X/Y)")]
		x_dpi: Option<u16>,
		#[arg(short = 'y', long, help = "Y-axis DPI (for split X/Y)")]
		y_dpi: Option<u16>,
	},

	#[command(about = "Set number of active DPI levels")]
	DpiLevels {
		#[arg(help = "Number of DPI levels to cycle through (1-4)")]
		count: u8,
	},

	#[command(about = "Set polling rate")]
	Rate {
		#[arg(
			value_name = "HZ",
			help = "Polling rate in Hz: 1000, 2000, 4000, or 8000"
		)]
		value: u32,
	},

	#[command(about = "Set lift-off distance")]
	Lod {
		#[arg(value_name = "MM", help = "Lift-off distance in mm: 1 or 2")]
		value: u8,
	},

	#[command(about = "Toggle angle snapping")]
	AngleSnap { value: OnOff },

	#[command(about = "Toggle ripple control")]
	Ripple { value: OnOff },

	#[command(about = "Toggle motion sync")]
	MotionSync { value: OnOff },

	#[command(about = "Configure filters")]
	Filter {
		#[command(subcommand)]
		filter: FilterCommand,
	},

	#[command(about = "Set debounce/multiclick value for a button")]
	Debounce {
		#[arg(help = BUTTON_HELP)]
		button: ButtonArg,
		#[arg(help = "Debounce value (0-25)")]
		value: u8,
	},

	#[command(about = "Set SPDT mode for the left or right button")]
	Spdt {
		#[arg(help = "Button: left, right, or a number 1-2")]
		button: ButtonArg,
		mode: SpdtValue,
	},

	#[command(
		about = "Remap a button",
		after_help = "Examples:\n  \
			egctl bind forward key f5\n  \
			egctl bind back media play-pause\n  \
			egctl bind middle scroll up\n  \
			egctl bind 6 cpi 400\n  \
			egctl bind 7 disable"
	)]
	Bind {
		#[arg(help = BUTTON_HELP)]
		button: ButtonArg,
		#[command(subcommand)]
		action: BindAction,
	},

	#[command(about = "List key names usable with 'bind <BUTTON> key'")]
	Keys,

	#[command(about = "Factory reset the mouse")]
	Reset,

	#[command(about = "Dump raw config as hex")]
	Dump,

	#[command(about = "Debug: enumerate HID devices")]
	Debug,
}

#[derive(Subcommand)]
pub enum FilterCommand {
	#[command(about = "Toggle slamclick filter")]
	Slamclick { value: OnOff },
	#[command(about = "Toggle jitter filter")]
	Jitter { value: OnOff },
}

#[derive(Subcommand)]
pub enum BindAction {
	#[command(about = "Map to a mouse button")]
	Mouse { key: MouseKeyArg },
	#[command(about = "Map to scroll wheel")]
	Scroll { direction: ScrollArg },
	#[command(about = "Map to a keyboard key or combo")]
	Key {
		#[arg(
			value_name = "KEY",
			help = "Key name or combo (e.g. a, f5, ctrl+c; see 'egctl keys') or HID usage code (e.g. 0x14)"
		)]
		key: KeyArg,
	},
	#[command(about = "Map to CPI level cycling")]
	CpiLoop,
	#[command(about = "Map to a CPI shift to a specific DPI while held")]
	Cpi {
		#[arg(help = "Target DPI while the button is held (50-26000)")]
		dpi: u16,
	},
	#[command(about = "Map to a media key")]
	Media { key: MediaKeyArg },
	#[command(about = "Disable the button")]
	Disable,
}

#[derive(Clone, Copy)]
pub struct ButtonArg(pub usize);

impl ButtonArg {
	pub fn name(self) -> &'static str {
		match self.0 {
			0 => "left",
			1 => "right",
			2 => "middle",
			3 => "forward",
			4 => "back",
			5 => "button 6",
			_ => "button 7",
		}
	}
}

impl FromStr for ButtonArg {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let idx = match s.to_ascii_lowercase().as_str() {
			"left" => Some(0),
			"right" => Some(1),
			"middle" => Some(2),
			"forward" => Some(3),
			"back" | "backward" => Some(4),
			other => match other.parse::<usize>() {
				Ok(n) if (1..=BUTTON_COUNT).contains(&n) => Some(n - 1),
				_ => None,
			},
		};
		idx.map(Self).ok_or_else(|| {
			format!("unknown button '{s}'. use left, right, middle, forward, back, or a number 1-{BUTTON_COUNT}")
		})
	}
}

#[derive(Clone, Copy)]
pub struct KeyArg {
	pub modifiers: u8,
	pub code: u8,
}

fn parse_key_part(s: &str) -> Option<u8> {
	if let Some(code) = key_code(s) {
		return Some(code);
	}
	if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
		u8::from_str_radix(hex, 16).ok()
	} else {
		s.parse().ok()
	}
}

impl FromStr for KeyArg {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let err = || {
			format!(
				"unknown key '{s}'. use a key name (e.g. a, enter, f5), a combo (e.g. ctrl+c), or a HID usage code (e.g. 0x14). run 'egctl keys' for the full list"
			)
		};
		let mut modifiers = 0u8;
		let parts: Vec<&str> = s.split('+').collect();
		let (last, mods) = parts.split_last().ok_or_else(err)?;
		for part in mods {
			modifiers |= modifier_bit(part).ok_or_else(err)?;
		}
		if let Some(bit) = modifier_bit(last) {
			return Ok(Self {
				modifiers: modifiers | bit,
				code: 0,
			});
		}
		let code = parse_key_part(last).ok_or_else(err)?;
		Ok(Self { modifiers, code })
	}
}

#[derive(Clone, ValueEnum)]
pub enum OnOff {
	On,
	Off,
}

impl OnOff {
	pub fn as_bool(&self) -> bool {
		matches!(self, Self::On)
	}
}

#[derive(Clone, ValueEnum)]
pub enum SpdtValue {
	Off,
	Safe,
	Speed,
}

#[derive(Clone, ValueEnum)]
pub enum MouseKeyArg {
	Left,
	Right,
	Middle,
	Back,
	Forward,
}

#[derive(Clone, ValueEnum)]
pub enum ScrollArg {
	Up,
	Down,
}

#[derive(Clone, ValueEnum)]
pub enum MediaKeyArg {
	PlayPause,
	Next,
	Previous,
	Mute,
	VolumeUp,
	VolumeDown,
	Browser,
	Explorer,
}
