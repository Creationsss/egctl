use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
	name = "egctl",
	about = "Configure the Endgame Gear OP1 8k v2 mouse on Linux"
)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
	#[command(about = "Show firmware version and all current settings")]
	Info,

	#[command(about = "Set DPI for a CPI level")]
	Dpi {
		value: Option<u16>,
		#[arg(short, long, default_value_t = 1, help = "CPI level (1-4)")]
		level: u8,
		#[arg(short = 'x', long, help = "X-axis DPI (for split X/Y)")]
		x_dpi: Option<u16>,
		#[arg(short = 'y', long, help = "Y-axis DPI (for split X/Y)")]
		y_dpi: Option<u16>,
	},

	#[command(about = "Set number of active DPI levels (1-4)")]
	DpiLevels { count: u8 },

	#[command(about = "Set polling rate (1000/2000/4000/8000)")]
	Rate { value: u32 },

	#[command(about = "Set lift-off distance")]
	Lod { value: u8 },

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

	#[command(about = "Set debounce/multiclick value for a button (0-25)")]
	Debounce { button: u8, value: u8 },

	#[command(about = "Set SPDT mode for a button")]
	Spdt { button: u8, mode: SpdtValue },

	#[command(about = "Remap a button")]
	Bind {
		button: u8,
		#[command(subcommand)]
		action: BindAction,
	},

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
	#[command(about = "Map to a keyboard key (HID usage code)")]
	Key { code: u8 },
	#[command(about = "Map to CPI level cycling")]
	CpiLoop,
	#[command(about = "Map to a specific CPI level (1-4)")]
	Cpi { level: u8 },
	#[command(about = "Map to a media key")]
	Media { key: MediaKeyArg },
	#[command(about = "Disable the button")]
	Disable,
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
