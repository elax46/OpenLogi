# Configuration

How OpenLogi stores its settings. For install and usage, see the
[README](../README.md).

Config is a TOML file, read on startup and written atomically on change:

- macOS & Linux: `$XDG_CONFIG_HOME/openlogi/config.toml` (default `~/.config/openlogi/config.toml`)
- Windows: `%USERPROFILE%\.config\openlogi\config.toml`

Per-device settings are keyed by the HID++ identifier (e.g. `2b042` for an
MX Master 4): `button_bindings`, `per_app_bindings` (keyed by bundle id such as
`com.microsoft.VSCode`), `gesture_bindings`, and `dpi_presets`. The app-wide
`[app_settings]` block holds `launch_at_login` and `check_for_updates` — both
default off and are edited directly in the file (there's no settings UI yet).

```toml
schema_version = 1
selected_device = "2b042"

[app_settings]
launch_at_login = true

[devices.2b042]
dpi_presets = [800, 1600, 3200]

[devices.2b042.button_bindings]
Back = "BrowserBack"
Forward = "BrowserForward"

# Per-app overlay: Back becomes Undo only while VS Code is frontmost.
[devices.2b042.per_app_bindings."com.microsoft.VSCode"]
Back = "Undo"
```
