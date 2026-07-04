# egctl

Configure the Endgame Gear OP1 8k v2 mouse on Linux via USB HID.

## Features

- Read and display all current settings
- Set DPI per level, with optional split X/Y
- Set polling rate (1000/2000/4000/8000 Hz)
- Configure lift-off distance, angle snapping, ripple control, motion sync
- Remap buttons to mouse keys, keyboard keys, media keys, scroll, or CPI switching
- Set per-button debounce and SPDT mode
- Toggle slamclick and jitter filters
- Factory reset

## Install

Download a prebuilt binary from [releases](https://heliopolis.live/creations/egctl/-/releases).

Or build from source:

```
cargo install --path .
```

## Setup

Install udev rules for non-root access:

```
sudo cp 60-endgamegear.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Unplug and replug the mouse after installing rules.

## Usage

```
egctl info
egctl dpi 1600
egctl dpi 800 --level 2
egctl dpi -x 800 -y 1600
egctl dpi-levels 2
egctl rate 4000
egctl lod 2
egctl angle-snap on
egctl ripple off
egctl motion-sync on
egctl filter slamclick on
egctl filter jitter off
egctl debounce left 8
egctl spdt left speed
egctl bind forward key f5
egctl bind back media play-pause
egctl bind middle scroll up
egctl bind 6 cpi 400
egctl bind 7 disable
egctl keys
egctl reset
egctl dump
```

Buttons are given by name (`left`, `right`, `middle`, `forward`, `back`) or number (1-7).
Keyboard keys are given by name (`a`, `enter`, `f5`, ... — see `egctl keys`), as a combo
(`ctrl+c`, `ctrl+shift+esc`), or as a raw HID usage code (`0x14`).

## License

AGPL-3.0-or-later
