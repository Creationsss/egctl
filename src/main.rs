mod cli;
mod device;
mod protocol;
mod types;

use std::io::{self, Write};

use anyhow::{bail, Result};
use clap::Parser;

use cli::*;
use device::Device;
use protocol::BUTTON_COUNT;
use types::*;

fn main() -> Result<()> {
	let cli = Cli::parse();

	match cli.command {
		Commands::Info => cmd_info(),
		Commands::Dpi {
			value,
			level,
			x_dpi,
			y_dpi,
		} => cmd_dpi(value, level, x_dpi, y_dpi),
		Commands::DpiLevels { count } => cmd_dpi_levels(count),
		Commands::Rate { value } => cmd_rate(value),
		Commands::Lod { value } => cmd_lod(value),
		Commands::AngleSnap { value } => {
			modify(|c| c.angle_snapping = value.as_bool())?;
			println!("Angle snapping: {}", on_off(value.as_bool()));
			Ok(())
		}
		Commands::Ripple { value } => {
			modify(|c| c.ripple_control = value.as_bool())?;
			println!("Ripple control: {}", on_off(value.as_bool()));
			Ok(())
		}
		Commands::MotionSync { value } => {
			modify(|c| c.motion_sync = value.as_bool())?;
			println!("Motion sync: {}", on_off(value.as_bool()));
			Ok(())
		}
		Commands::Filter { filter } => cmd_filter(filter),
		Commands::Debounce { button, value } => cmd_debounce(button, value),
		Commands::Spdt { button, mode } => cmd_spdt(button, mode),
		Commands::Bind { button, action } => cmd_bind(button, action),
		Commands::Reset => cmd_reset(),
		Commands::Dump => cmd_dump(),
		Commands::Debug => device::debug_enumerate(),
	}
}

fn modify(f: impl FnOnce(&mut MouseConfig)) -> Result<()> {
	let dev = Device::open()?;
	let mut config = dev.read_config()?;
	f(&mut config);
	dev.write_config(&config)
}

fn validate_button(b: u8) -> Result<usize> {
	if b < 1 || b > BUTTON_COUNT as u8 {
		bail!("invalid button index {b}. must be 1-{BUTTON_COUNT}");
	}
	Ok((b - 1) as usize)
}

fn cmd_info() -> Result<()> {
	let dev = Device::open()?;

	let (major, minor) = dev.get_firmware_version()?;
	println!("Firmware: {major:02x}.{minor:02x}");
	println!();

	let config = dev.read_config()?;
	print!("{config}");
	Ok(())
}

fn cmd_dpi(value: Option<u16>, level: u8, x_dpi: Option<u16>, y_dpi: Option<u16>) -> Result<()> {
	if level < 1 || level > 4 {
		bail!("invalid DPI level {level}. must be 1-4");
	}
	let idx = (level - 1) as usize;

	let (xy_split, x, y) = match (value, x_dpi, y_dpi) {
		(Some(v), None, None) => {
			let dpi = validate_dpi(v)?;
			(false, dpi, dpi)
		}
		(None, Some(x), Some(y)) => (true, validate_dpi(x)?, validate_dpi(y)?),
		(None, Some(x), None) => {
			let dpi = validate_dpi(x)?;
			(false, dpi, dpi)
		}
		(None, None, None) => {
			let dev = Device::open()?;
			let config = dev.read_config()?;
			let cpi = &config.cpis[idx];
			if cpi.xy_split {
				println!("CPI {level}: X={} Y={}", cpi.x, cpi.y);
			} else {
				println!("CPI {level}: {}", cpi.x);
			}
			return Ok(());
		}
		_ => bail!("conflicting DPI arguments. use VALUE, or --x X --y Y"),
	};

	modify(|c| {
		c.cpis[idx].xy_split = xy_split;
		c.cpis[idx].x = x;
		c.cpis[idx].y = y;
	})?;

	if xy_split {
		println!("CPI {level}: X={x} Y={y}");
	} else {
		println!("CPI {level}: {x}");
	}
	Ok(())
}

fn cmd_dpi_levels(count: u8) -> Result<()> {
	if count < 1 || count > 4 {
		bail!("invalid DPI level count {count}. must be 1-4");
	}
	modify(|c| c.cpi_levels = count)?;
	println!("Active CPI levels: {count}");
	Ok(())
}

fn cmd_rate(hz: u32) -> Result<()> {
	let rate = PollingRate::from_hz(hz)?;
	modify(|c| c.polling_rate = rate)?;
	println!("Polling rate: {rate}");
	Ok(())
}

fn cmd_lod(value: u8) -> Result<()> {
	modify(|c| c.lod = value)?;
	println!("Lift-off distance: {value}");
	Ok(())
}

fn cmd_filter(filter: FilterCommand) -> Result<()> {
	match filter {
		FilterCommand::Slamclick { value } => {
			let v = value.as_bool();
			modify(|c| c.slamclick_filter = v)?;
			println!("Slamclick filter: {}", on_off(v));
		}
		FilterCommand::Jitter { value } => {
			let v = value.as_bool();
			modify(|c| c.jitter_filter = v)?;
			println!("Jitter filter: {}", on_off(v));
		}
	}
	Ok(())
}

fn cmd_debounce(button: u8, value: u8) -> Result<()> {
	let idx = validate_button(button)?;
	if value > 25 {
		bail!("invalid debounce value {value}. must be 0-25");
	}
	modify(|c| c.buttons[idx].multiclick = value)?;
	println!("Button {button} debounce: {value}");
	Ok(())
}

fn cmd_spdt(button: u8, mode: SpdtValue) -> Result<()> {
	let idx = validate_button(button)?;
	let spdt = match mode {
		SpdtValue::Off => SpdtMode::Off,
		SpdtValue::Safe => SpdtMode::Safe,
		SpdtValue::Speed => SpdtMode::Speed,
	};
	modify(|c| c.buttons[idx].spdt = spdt)?;
	println!("Button {button} SPDT: {spdt}");
	Ok(())
}

fn cmd_bind(button: u8, action: BindAction) -> Result<()> {
	let idx = validate_button(button)?;
	let mapping = match action {
		BindAction::Mouse { key } => MappingType::Mouse(match key {
			MouseKeyArg::Left => MouseKey::Left,
			MouseKeyArg::Right => MouseKey::Right,
			MouseKeyArg::Middle => MouseKey::Middle,
			MouseKeyArg::Back => MouseKey::Back,
			MouseKeyArg::Forward => MouseKey::Forward,
		}),
		BindAction::Scroll { direction } => MappingType::Scroll(match direction {
			ScrollArg::Up => ScrollDirection::Up,
			ScrollArg::Down => ScrollDirection::Down,
		}),
		BindAction::Key { code } => MappingType::Keyboard(code),
		BindAction::CpiLoop => MappingType::CpiLoop,
		BindAction::Cpi { level } => {
			if level < 1 || level > 4 {
				bail!("invalid CPI level {level}. must be 1-4");
			}
			MappingType::Cpi(level - 1)
		}
		BindAction::Media { key } => MappingType::Media(match key {
			MediaKeyArg::PlayPause => MediaKey::PlayPause,
			MediaKeyArg::Next => MediaKey::Next,
			MediaKeyArg::Previous => MediaKey::Previous,
			MediaKeyArg::Mute => MediaKey::Mute,
			MediaKeyArg::VolumeUp => MediaKey::VolumeUp,
			MediaKeyArg::VolumeDown => MediaKey::VolumeDown,
			MediaKeyArg::Browser => MediaKey::Browser,
			MediaKeyArg::Explorer => MediaKey::Explorer,
		}),
		BindAction::Disable => MappingType::Disable,
	};

	modify(|c| c.buttons[idx].mapping = mapping)?;
	println!("Button {button}: {mapping}");
	Ok(())
}

fn cmd_reset() -> Result<()> {
	print!("Factory reset the mouse? [y/N] ");
	io::stdout().flush().ok();
	let mut input = String::new();
	io::stdin().read_line(&mut input).ok();
	if !input.trim().eq_ignore_ascii_case("y") {
		println!("Cancelled.");
		return Ok(());
	}

	let dev = Device::open()?;
	dev.factory_reset()?;
	println!("Factory reset complete.");
	Ok(())
}

fn cmd_dump() -> Result<()> {
	let dev = Device::open()?;
	let raw = dev.read_raw()?;

	for (i, chunk) in raw.chunks(16).enumerate() {
		print!("{:04x}:  ", i * 16);
		for (j, byte) in chunk.iter().enumerate() {
			if j == 8 {
				print!(" ");
			}
			print!("{byte:02x} ");
		}
		println!();
	}
	Ok(())
}
