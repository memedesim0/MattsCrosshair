# Matt's Crosshair

A simple desktop crosshair overlay for Windows written in Rust.

## Usage

Run the executable located at:
```text
target\release\matts_crosshair.exe
```

Once started, the app stays in the Windows system tray. Use the tray icon to open the settings window or close the program.

## Build from source

Requires [Rust](https://rustup.rs/).

```bash
cargo build --release
```

## Anti-Cheat

This app renders a standard transparent desktop overlay and does not read, modify, or hook into game memory. Still, always check a game's terms of service before using third-party overlays.

## License

This project is free and open-source software licensed under the **GNU General Public License v3.0**. You are free to use, modify, and distribute it under the terms of the GPLv3. Any derivative work must also remain open-source under the same license. See the [LICENSE](LICENSE) file for details.
