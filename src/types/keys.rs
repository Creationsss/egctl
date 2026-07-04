pub const KEY_NAMES: &[(u8, &str)] = &[
	(0x04, "a"),
	(0x05, "b"),
	(0x06, "c"),
	(0x07, "d"),
	(0x08, "e"),
	(0x09, "f"),
	(0x0a, "g"),
	(0x0b, "h"),
	(0x0c, "i"),
	(0x0d, "j"),
	(0x0e, "k"),
	(0x0f, "l"),
	(0x10, "m"),
	(0x11, "n"),
	(0x12, "o"),
	(0x13, "p"),
	(0x14, "q"),
	(0x15, "r"),
	(0x16, "s"),
	(0x17, "t"),
	(0x18, "u"),
	(0x19, "v"),
	(0x1a, "w"),
	(0x1b, "x"),
	(0x1c, "y"),
	(0x1d, "z"),
	(0x1e, "1"),
	(0x1f, "2"),
	(0x20, "3"),
	(0x21, "4"),
	(0x22, "5"),
	(0x23, "6"),
	(0x24, "7"),
	(0x25, "8"),
	(0x26, "9"),
	(0x27, "0"),
	(0x28, "enter"),
	(0x29, "esc"),
	(0x2a, "backspace"),
	(0x2b, "tab"),
	(0x2c, "space"),
	(0x2d, "minus"),
	(0x2e, "equal"),
	(0x2f, "left-bracket"),
	(0x30, "right-bracket"),
	(0x31, "backslash"),
	(0x33, "semicolon"),
	(0x34, "apostrophe"),
	(0x35, "grave"),
	(0x36, "comma"),
	(0x37, "period"),
	(0x38, "slash"),
	(0x39, "caps-lock"),
	(0x3a, "f1"),
	(0x3b, "f2"),
	(0x3c, "f3"),
	(0x3d, "f4"),
	(0x3e, "f5"),
	(0x3f, "f6"),
	(0x40, "f7"),
	(0x41, "f8"),
	(0x42, "f9"),
	(0x43, "f10"),
	(0x44, "f11"),
	(0x45, "f12"),
	(0x46, "print-screen"),
	(0x47, "scroll-lock"),
	(0x48, "pause"),
	(0x49, "insert"),
	(0x4a, "home"),
	(0x4b, "page-up"),
	(0x4c, "delete"),
	(0x4d, "end"),
	(0x4e, "page-down"),
	(0x4f, "right"),
	(0x50, "left"),
	(0x51, "down"),
	(0x52, "up"),
	(0x53, "num-lock"),
	(0x54, "kp-slash"),
	(0x55, "kp-asterisk"),
	(0x56, "kp-minus"),
	(0x57, "kp-plus"),
	(0x58, "kp-enter"),
	(0x59, "kp-1"),
	(0x5a, "kp-2"),
	(0x5b, "kp-3"),
	(0x5c, "kp-4"),
	(0x5d, "kp-5"),
	(0x5e, "kp-6"),
	(0x5f, "kp-7"),
	(0x60, "kp-8"),
	(0x61, "kp-9"),
	(0x62, "kp-0"),
	(0x63, "kp-period"),
	(0x65, "menu"),
];

pub const MODIFIER_NAMES: &[(u8, &str)] = &[
	(0x01, "ctrl"),
	(0x02, "shift"),
	(0x04, "alt"),
	(0x08, "meta"),
	(0x10, "right-ctrl"),
	(0x20, "right-shift"),
	(0x40, "right-alt"),
	(0x80, "right-meta"),
];

pub fn key_code(name: &str) -> Option<u8> {
	let lower = name.to_ascii_lowercase();
	let canonical = match lower.as_str() {
		"escape" => "esc",
		"return" => "enter",
		"del" => "delete",
		"ins" => "insert",
		"pgup" => "page-up",
		"pgdn" | "pgdown" => "page-down",
		"caps" | "capslock" => "caps-lock",
		"numlock" => "num-lock",
		"dot" | "." => "period",
		"-" => "minus",
		"=" => "equal",
		"[" => "left-bracket",
		"]" => "right-bracket",
		"\\" => "backslash",
		";" => "semicolon",
		"'" => "apostrophe",
		"`" => "grave",
		"," => "comma",
		"/" => "slash",
		other => other,
	};
	KEY_NAMES
		.iter()
		.find(|(_, n)| *n == canonical)
		.map(|(c, _)| *c)
}

pub fn key_name(code: u8) -> Option<&'static str> {
	KEY_NAMES.iter().find(|(c, _)| *c == code).map(|(_, n)| *n)
}

pub fn modifier_bit(name: &str) -> Option<u8> {
	let lower = name.to_ascii_lowercase();
	let canonical = match lower.as_str() {
		"control" | "lctrl" | "left-ctrl" => "ctrl",
		"lshift" | "left-shift" => "shift",
		"lalt" | "left-alt" => "alt",
		"super" | "win" | "gui" | "cmd" | "lmeta" | "left-meta" => "meta",
		"rctrl" => "right-ctrl",
		"rshift" => "right-shift",
		"altgr" | "ralt" => "right-alt",
		"rmeta" | "rwin" | "rsuper" => "right-meta",
		other => other,
	};
	MODIFIER_NAMES
		.iter()
		.find(|(_, n)| *n == canonical)
		.map(|(b, _)| *b)
}

pub fn format_key(modifiers: u8, code: u8) -> String {
	let mut parts: Vec<String> = MODIFIER_NAMES
		.iter()
		.filter(|(b, _)| modifiers & b != 0)
		.map(|(_, n)| n.to_string())
		.collect();
	if code != 0 || parts.is_empty() {
		match key_name(code) {
			Some(name) => parts.push(name.to_string()),
			None => parts.push(format!("0x{code:02x}")),
		}
	}
	parts.join("+")
}
