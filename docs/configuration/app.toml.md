# app.toml

## Default application configuration

```toml
# `mouse_support` enables mouse support in the terminal.
mouse_support = true
# `paste_support` enables paste support in the terminal.
paste_support = true
# `frame_rate` is the frame rate of the terminal in frames per second.
# The suggested frame rate are:
# - 120.0 for ultra smooth animations
# - 60.0 for smooth animations
# - 30.0 for normal animations
# - 15.0 for slow animations
frame_rate = 60.0
# `show_status_bar` enables the status bar at the bottom of the terminal.
show_status_bar = true
# `show_title_bar` enables the title bar at the top of the terminal.
show_title_bar = true
# `theme_enable` enables the theme.
theme_enable = true
# `theme_filename` is the name of the file that contains the theme.
# This file must be in the configuration directory.
theme_filename = "theme.toml"
# `take_api_id_from_telegram_config` enables taking the API_ID from the Telegram configuration file
# or from the environment variable `API_ID`.
take_api_id_from_telegram_config = true
# `take_api_hash_from_telegram_config` enables taking the API_HASH from the Telegram configuration file
# or from the environment variable `API_HASH`.
take_api_hash_from_telegram_config = true
```

## Custom configuration

### How create a custom configuration file

`tgt` by default reads its **default** configurations from:
- Linux: `/home/<name>/.tgt/config/`
- macOS: `/Users/<name>/.tgt/config/`
- Windows: `C:\Users\<name>\.tgt\config/`

We suggest you to not modify this file, but to create your own **custom** configuration file in the following directories (in order of precedence):

- `$TGT_CONFIG_DIR` (if set)
- `$HOME/.config/tgt/` (for Linux and macOS) and `C:\Users\<name>\AppData\Roaming\tgt\` (for Windows)

Reading configurations from the following directories will override the fields defined in the default configuration files.
It means that the fields that are not present in the custom configuration will be taken from the default configuration, while the fields that are present in the custom configuration will override the default configuration.
Note that after the finding the first configuration file, `tgt` stops looking for more configurations, it is short-circuited.

### Example of a custom application configuration

Example of `app.toml`:

```toml
show_status_bar = false
show_title_bar = false
```
