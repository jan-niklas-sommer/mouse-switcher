# Mouse Switcher

Lightweight Windows system-tray app to quickly switch mouse sensitivity between normal and gaming profiles. Single `.exe`, no installation, no dependencies.

## Features

- **Two profiles**: Normal (default Windows speed) and Gaming (lower speed, no acceleration)
- **System tray icon** with right-click menu to switch profiles
- **Global hotkey** (`Ctrl+Alt+M`) to toggle between profiles instantly
- **Configurable** via `settings.toml` next to the `.exe`
- **Single binary** — just download and run

## Usage

1. Download the latest release ZIP
2. Extract `mouse-switcher.exe` and `settings.toml` to any folder
3. Run `mouse-switcher.exe`
4. Use `Ctrl+Alt+M` or right-click the tray icon to switch profiles

## Configuration

Edit `settings.toml` (right-click tray icon → Open Settings):

```toml
[normal]
speed = 10                    # 1-20, Windows default is 10
enhance_precision = true      # Mouse acceleration ON

[gaming]
speed = 4                     # Lower = slower mouse
enhance_precision = false     # Mouse acceleration OFF (better for gaming)

[hotkey]
toggle = "Ctrl+Alt+M"        # Hotkey to toggle profiles
```

### Mouse Speed Values

| Value | Description |
|-------|-------------|
| 1     | Slowest |
| 10    | Windows default |
| 20    | Fastest |

### Hotkey Format

Use modifier keys + a key: `Ctrl+Alt+M`, `Ctrl+Shift+1`, `Alt+G`, etc.

## Autostart (optional)

Create a shortcut to `mouse-switcher.exe` in your Windows Startup folder:
1. Press `Win+R`, type `shell:startup`, press Enter
2. Create a shortcut to `mouse-switcher.exe` in that folder

## Building from source

```bash
cargo build --release
```

Requires Rust and the MSVC build tools (Visual Studio Build Tools).

## License

MIT
