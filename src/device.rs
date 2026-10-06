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

		let have_config_iface = paths.iter().any(|p| p.0 == 1);

		let mut last_err = None;
		for (iface, path) in &paths {
			if have_config_iface && *iface < 1 {
				continue;
			}
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
					last_err = Some(msg);
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

		if let Some(msg) = last_err {
			bail!("failed to open config interface (VID:{VID:#06x} PID:{PID:#06x}): {msg}");
		}

		bail!("device not found (VID:{VID:#06x} PID:{PID:#06x}). is the mouse plugged in?")
	}

	pub fn read_config(&self) -> Result<MouseConfig> {
		let mut buf = [0u8; CONFIG_SIZE];
		let n = self.query(OP_LOAD_CONFIG, &mut buf[..CONFIG_SIZE - 3], |b| {
			(1..=CPI_COUNT as u8).contains(&b[OFF_CPI_LEVELS])
		})?;
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
		let mut resp = [0u8; COMMAND_SIZE];
		let n = self.query(OP_GET_FW_VERSION, &mut resp[..COMMAND_SIZE - 1], |b| {
			b[FW_VERSION_MAJOR] != 0 || b[FW_VERSION_MINOR] != 0
		})?;
		if n < COMMAND_SIZE - 2 {
			bail!(
				"firmware version read: expected {} bytes, got {n}",
				COMMAND_SIZE - 2
			);
		}

		Ok((resp[FW_VERSION_MAJOR], resp[FW_VERSION_MINOR]))
	}

	fn query(&self, op: u16, buf: &mut [u8], valid: impl Fn(&[u8]) -> bool) -> Result<usize> {
		let mut cmd = [0u8; COMMAND_SIZE];
		let op = op.to_le_bytes();
		cmd[0] = op[0];
		cmd[1] = op[1];

		for _ in 0..QUERY_ATTEMPTS {
			self.hid.send_feature_report(&cmd)?;
			thread::sleep(QUERY_DELAY);

			buf.fill(0);
			buf[0] = REPORT_ID_READ;
			let n = self.hid.get_feature_report(buf)?;
			if valid(buf) {
				return Ok(n);
			}
		}

		bail!(
			"mouse returned an incomplete response after {QUERY_ATTEMPTS} attempts \
			 (status byte {:#04x}). refusing to continue",
			buf[1]
		)
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
