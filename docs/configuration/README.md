# Configuration

`tgt` by default reads its **default** configurations from:
- Linux: `/home/<name>/.tgt/config/`
- macOS: `/Users/<name>/.tgt/config/`
- Windows: `C:\Users\<name>\.tgt\config/`

We suggest you to not modify these files, but to create your own **custom** configuration files in the following directories (in order of precedence):

- `$TGT_CONFIG_DIR` (if set)
- `$HOME/.config/tgt/config` (for Linux and macOS) and `C:\Users\<name>\AppData\Roaming\tgt\config` (for Windows)

Reading configurations from the following directories will override the fields defined in the default configuration files.
It means that the fields that are not present in the custom configuration will be taken from the default configuration, while the fields that are present in the custom configuration will override the default configuration.
Note that after the finding the first configuration file, `tgt` stops looking for more configurations, it is short-circuited.

## Configuration Files

In the configuration directory, `tgt` looks for the following files:

- `app.toml` for application configuration (see [App Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/app.toml.md))
- `logger.toml` for logger configuration (see [Logger Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/logger.toml.md))
- `telegram.toml` for Telegram configuration (see [Telegram Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/telegram.toml.md))
- `theme.toml` for theme configuration (see [Theme Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/theme.toml.md))
- `keymap.toml` for keymap configuration (see [Keymap Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/keymap.toml.md))

### Theme Files

Theme files are stored in the `themes/` subdirectory within your configuration directory. Multiple theme files can exist, and `tgt` will automatically discover all `.toml` files in the `themes/` directory. The default theme is `themes/theme.toml`.

**Example theme directory structure:**
```
~/.config/tgt/config/
├── app.toml
├── logger.toml
├── telegram.toml
├── keymap.toml
└── themes/
    ├── theme.toml          # Default theme
    ├── monokai.toml
    ├── nord.toml
    ├── gruvbox.toml
    └── my_custom_theme.toml
```

You can switch between themes using the theme switcher (see keymap configuration) or by setting `theme_filename = "themes/<theme_name>.toml"` in your `app.toml` file.
