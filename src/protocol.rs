pub const VID: u16 = 0x3367;
pub const PID: u16 = 0x1978;

pub const COMMAND_SIZE: usize = 64;
pub const CONFIG_SIZE: usize = 1041;

pub const CPI_COUNT: usize = 4;
pub const BUTTON_COUNT: usize = 7;

pub const OP_STORE_CONFIG: u16 = 0x11a0;
pub const OP_LOAD_CONFIG: u16 = 0x12a1;
pub const OP_FACTORY_RESET: u16 = 0x13a1;
pub const OP_GET_FW_VERSION: u16 = 0x02a1;

pub const REPORT_ID_READ: u8 = 0xa1;

pub const FILTER_SLAMCLICK: u8 = 0x01;
pub const FILTER_JITTER: u8 = 0x10;

pub const OFF_POLLING_DIVIDER: usize = 21;
pub const OFF_FILTER_FLAGS: usize = 22;
pub const OFF_LOD: usize = 25;
pub const OFF_ANGLE_SNAPPING: usize = 26;
pub const OFF_RIPPLE_CONTROL: usize = 27;
pub const OFF_MOTION_SYNC: usize = 28;
pub const OFF_CPI_LEVELS: usize = 30;
pub const OFF_CPIS: usize = 51;
pub const OFF_BUTTONS: usize = 77;

pub const OFF_GLASS_MODE: usize = 127;
pub const OFF_FORCE_MAX_FPS: usize = 129;

pub const CPI_STRUCT_SIZE: usize = 5;
pub const BUTTON_STRUCT_SIZE: usize = 7;

pub const CPI_MIN: u16 = 50;
pub const CPI_MAX: u16 = 26000;
pub const CPI_STEP: u16 = 50;

pub const SPDT_SAFE: u8 = 0xF0;
pub const SPDT_SPEED: u8 = 0xF1;
pub const SPDT_THRESHOLD: u8 = 0xF0;

pub const MAP_MOUSE: i8 = 0;
pub const MAP_SCROLL: i8 = 1;
pub const MAP_KEYBOARD: i8 = 2;
pub const MAP_CPI_LOOP: i8 = 9;
pub const MAP_CPI: i8 = 12;
pub const MAP_MEDIA: i8 = 32;
pub const MAP_DISABLE: i8 = -1;

pub const MOUSE_LEFT: u8 = 1;
pub const MOUSE_RIGHT: u8 = 2;
pub const MOUSE_MIDDLE: u8 = 4;
pub const MOUSE_BACK: u8 = 8;
pub const MOUSE_FORWARD: u8 = 16;

pub const MEDIA_PLAY_PAUSE: u8 = 0xCD;
pub const MEDIA_NEXT: u8 = 0xB5;
pub const MEDIA_PREVIOUS: u8 = 0xB6;
pub const MEDIA_MUTE: u8 = 0xE2;
pub const MEDIA_VOLUME_UP: u8 = 0xE9;
pub const MEDIA_VOLUME_DOWN: u8 = 0xEA;
pub const MEDIA_BROWSER: u8 = 0x96;
pub const MEDIA_EXPLORER: u8 = 0x94;

pub const FW_VERSION_MAJOR: usize = 17;
pub const FW_VERSION_MINOR: usize = 18;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn report_ids_match_opcodes() {
		assert_eq!(OP_LOAD_CONFIG.to_le_bytes()[0], REPORT_ID_READ);
		assert_eq!(OP_FACTORY_RESET.to_le_bytes()[0], REPORT_ID_READ);
		assert_eq!(OP_GET_FW_VERSION.to_le_bytes()[0], REPORT_ID_READ);
	}

	#[test]
	fn cpi_offsets_fit_in_config() {
		let end = OFF_CPIS + CPI_COUNT * CPI_STRUCT_SIZE;
		assert!(end <= CONFIG_SIZE);
		assert_eq!(end, 71);
	}

	#[test]
	fn button_offsets_fit_in_config() {
		let end = OFF_BUTTONS + BUTTON_COUNT * BUTTON_STRUCT_SIZE;
		assert!(end <= CONFIG_SIZE);
		assert_eq!(end, 126);
	}
}
