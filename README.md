# Mouse Switcher

Lightweight Windows system-tray app to quickly switch and adjust mouse sensitivity between normal and gaming profiles. Single `.exe`, no installation, no dependencies.

## Features

- **Two profiles**: Normal (default Windows speed) and Gaming (lower speed, no acceleration)
- **In-game hotkeys**: Adjust sensitivity without leaving your game
- **System tray icon** with right-click menu for full control
- **Speed adjustment**: +1 / -1 steps via hotkey or tray menu (range 1-20)
- **Acceleration toggle**: Enable/disable "Enhance Pointer Precision" per profile
- **Auto-save**: All changes are saved to `settings.toml` instantly
- **Single binary** — just download and run, no installer needed

## Quick Start

1. Download the latest release ZIP from [Releases](https://github.com/jan-niklas-sommer/mouse-switcher/releases)
2. Extract `mouse-switcher.exe` and `settings.toml` to any folder
3. Run `mouse-switcher.exe`
4. Use hotkeys or right-click the tray icon to control

## Hotkeys

All hotkeys work globally, even when a game is in the foreground.

| Action | Default Hotkey |
|--------|---------------|
| Toggle Normal / Gaming profile | `Ctrl+Alt+M` |
| Increase mouse speed (+1) | `Ctrl+Alt+ArrowUp` |
| Decrease mouse speed (-1) | `Ctrl+Alt+ArrowDown` |

**Note**: In the `settings.toml` config file, key names use the `keyboard-types` format:
letters are `KeyA`-`KeyZ`, digits are `Digit0`-`Digit9`, arrows are `ArrowUp`/`ArrowDown`/`ArrowLeft`/`ArrowRight`.
The app auto-normalizes common names (e.g. `M` → `KeyM`, `5` → `Digit5`, `Up` → `ArrowUp`).

## Tray Menu

Right-click the tray icon for full control:

```
┌──────────────────────────────┐
│ ✓ Normal                     │  ← Switch to Normal profile
│   Gaming                     │  ← Switch to Gaming profile
│ ─────────────────────────────│
│   Speed: 10                  │  ← Current speed (info only)
│   Speed ▲  (+1)              │  ← Increase speed
│   Speed ▼  (-1)              │  ← Decrease speed
│ ✓ Acceleration: ON           │  ← Toggle mouse acceleration
│ ─────────────────────────────│
│   Toggle Profile             │  ← Quick toggle
│   Open Settings              │  ← Edit settings.toml in Notepad
│   Quit                       │
└──────────────────────────────┘
```

- **Speed ▲/▼** and **Acceleration** only affect the currently active profile
- Speed is clamped to range **1-20** (1 = slowest, 10 = Windows default, 20 = fastest)
- Speed ▲ is disabled at 20, Speed ▼ is disabled at 1
- All changes are auto-saved to `settings.toml`

## Configuration

Edit `settings.toml` (right-click tray → Open Settings, or edit manually):

```toml
[normal]
speed = 10                    # 1-20, Windows default is 10
enhance_precision = true      # Mouse acceleration ON

[gaming]
speed = 4                     # Lower = slower, better for FPS/aiming
enhance_precision = false     # Mouse acceleration OFF (raw input)

[hotkey]
toggle = "Ctrl+Alt+KeyM"
speed_up = "Ctrl+Alt+ArrowUp"
speed_down = "Ctrl+Alt+ArrowDown"
```

### Mouse Speed Values

| Value | Description |
|-------|-------------|
| 1     | Slowest |
| 4-6   | Typical gaming range |
| 10    | Windows default |
| 20    | Fastest |

### Mouse Acceleration ("Enhance Pointer Precision")

- **ON** (Normal): Windows moves the cursor faster when you move the mouse quickly. Good for desktop use.
- **OFF** (Gaming): 1:1 mouse movement — cursor distance is proportional to physical mouse movement. Essential for consistent aiming in FPS games.

### Hotkey Format

Hotkeys use modifier keys + a key name from the [`Code`](https://docs.rs/keyboard-types/latest/keyboard_types/enum.Code.html) enum. Examples:
- `Ctrl+Alt+KeyM` (letter M)
- `Ctrl+Shift+Digit1` (number 1)
- `Alt+KeyG` (letter G)
- `Ctrl+Alt+ArrowUp` / `Ctrl+Alt+ArrowDown` (arrow keys)

The app auto-normalizes shorthand names: `M` → `KeyM`, `5` → `Digit5`, `Up` → `ArrowUp`, `Space` → `Space`, etc.

Supported modifiers: `Ctrl`, `Alt`, `Shift`, `Super`

## Autostart (optional)

To start Mouse Switcher automatically with Windows:

1. Press `Win+R`, type `shell:startup`, press Enter
2. Create a shortcut to `mouse-switcher.exe` in that folder

## How it works

Mouse Switcher uses the Windows `SystemParametersInfoW` API to change:
- **Mouse speed** (`SPI_SETMOUSESPEED`): Changes the pointer speed slider value (1-20)
- **Mouse acceleration** (`SPI_SETMOUSE`): Toggles the "Enhance pointer precision" checkbox

These are the same settings found in Windows Settings → Mouse → Additional mouse settings → Pointer Options.

## Building from source

Requires Rust and Visual Studio Build Tools (MSVC).

```bash
cargo build --release
```

The `.exe` will be at `target/release/mouse-switcher.exe`.

## Tech Stack

- **Rust** — compiled to a single static binary (~1-3 MB)
- **Windows API** — `SystemParametersInfoW` for mouse settings
- **tray-icon** — system tray with native Windows menu
- **global-hotkey** — system-wide hotkey registration

## License

MIT
