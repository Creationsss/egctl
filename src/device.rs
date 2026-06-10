use std::thread;
use std::time::Duration;

use anyhow::{bail, Result};
use hidapi::HidApi;

use crate::protocol::*;
use crate::types::MouseConfig;

pub struct Device {
	_api: HidApi,
	hid: hidapi::HidDevice,
}

impl Device {
	pub fn open() -> Result<Self> {
		let api = HidApi::new()?;

		let mut paths = Vec::new();
		let mut permission_denied = false;

		for info in api.device_list() {
			if info.vendor_id() == VID && info.product_id() == PID {
				paths.push((info.interface_number(), info.path().to_owned()));
			}
		}

		if paths.is_empty() {
			bail!("device not found (VID:{VID:#06x} PID:{PID:#06x}). is the mouse plugged in?");
		}

		paths.sort_by_key(|p| std::cmp::Reverse(p.0));

		for (_iface, path) in &paths {
			match api.open_path(path) {
				Ok(hid) => return Ok(Self { _api: api, hid }),
				Err(e) => {
					let msg = format!("{e}");
					if msg.contains("Permission")
						|| msg.contains("EACCES")
						|| msg.contains("access")
					{
						permission_denied = true;
					}
				}
			}
		}

		if permission_denied {
			bail!(
				"permission denied opening HID device. install udev rules:\n  \
				 sudo cp 60-endgamegear.rules /etc/udev/rules.d/\n  \
				 sudo udevadm control --reload-rules && sudo udevadm trigger"
			);
		}

		bail!("device not found (VID:{VID:#06x} PID:{PID:#06x}). is the mouse plugged in?")
	}

	pub fn read_config(&self) -> Result<MouseConfig> {
		let mut cmd = [0u8; COMMAND_SIZE];
		let op = OP_LOAD_CONFIG.to_le_bytes();
		cmd[0] = op[0];
		cmd[1] = op[1];
		self.hid.send_feature_report(&cmd)?;

		let mut buf = [0u8; CONFIG_SIZE];
		buf[0] = REPORT_ID_READ;
		let n = self.hid.get_feature_report(&mut buf[..CONFIG_SIZE - 3])?;
		if n < CONFIG_SIZE - 4 {
			bail!("config read: expected {} bytes, got {n}", CONFIG_SIZE - 4);
		}

		Ok(MouseConfig::from_bytes(&buf))
	}

	pub fn write_config(&self, config: &MouseConfig) -> Result<()> {
		let mut buf = config.to_bytes();
		let op = OP_STORE_CONFIG.to_le_bytes();
		buf[0] = op[0];
		buf[1] = op[1];
		self.hid.send_feature_report(&buf)?;
		Ok(())
	}

	pub fn factory_reset(&self) -> Result<()> {
		let mut cmd = [0u8; COMMAND_SIZE];
		let op = OP_FACTORY_RESET.to_le_bytes();
		cmd[0] = op[0];
		cmd[1] = op[1];
		self.hid.send_feature_report(&cmd)?;
		thread::sleep(Duration::from_millis(150));
		Ok(())
	}

	pub fn get_firmware_version(&self) -> Result<(u8, u8)> {
		let mut cmd = [0u8; COMMAND_SIZE];
		let op = OP_GET_FW_VERSION.to_le_bytes();
		cmd[0] = op[0];
		cmd[1] = op[1];
		self.hid.send_feature_report(&cmd)?;

		let mut resp = [0u8; COMMAND_SIZE];
		resp[0] = REPORT_ID_READ;
		let n = self.hid.get_feature_report(&mut resp[..COMMAND_SIZE - 1])?;
		if n < COMMAND_SIZE - 2 {
			bail!(
				"firmware version read: expected {} bytes, got {n}",
				COMMAND_SIZE - 2
			);
		}

		Ok((resp[FW_VERSION_MAJOR], resp[FW_VERSION_MINOR]))
	}

	pub fn read_raw(&self) -> Result<[u8; CONFIG_SIZE]> {
		let config = self.read_config()?;
		Ok(config.raw)
	}
}

pub fn debug_enumerate() -> Result<()> {
	let api = HidApi::new()?;
	let mut count = 0;
	for info in api.device_list() {
		count += 1;
		println!(
			"  VID:{:#06x} PID:{:#06x} iface:{} usage:{:#06x} path:{:?}",
			info.vendor_id(),
			info.product_id(),
			info.interface_number(),
			info.usage_page(),
			info.path()
		);
	}
	println!("total HID devices enumerated: {count}");

	println!("\nattempting direct open(VID, PID)...");
	match api.open(VID, PID) {
		Ok(_) => println!("success!"),
		Err(e) => println!("failed: {e}"),
	}
	Ok(())
}
