# Configuration

`tgt` by default reads its default configurations from:
- Linux: `/home/<name>/.tgt/config`
- macOS: `/Users/<name>/.tgt/config`
- Windows: `C:\Users\<name>\.tgt\config`

We suggest you to not modify these files, but to create your own configuration files in the following directories (in order of precedence):
Reading configurations from the following directories will override the fields defined in the default configuration files:

- `$TGT_CONFIG_HOME` (if set)
- `$HOME/.config/tgt` (for Linux and macOS) and `C:\Users\<name>\AppData\Roaming\tgt` (for Windows)

Note that after the finding the first configuration file, `tgt` stops looking for more configurations, it is short-circuited.

## Configuration Files

In the configuration directory, `tgt` looks for the following files:

- `app.toml` for theme configuration (see [App Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/app.toml.md))
- `keymap.toml` for keymap configuration (see [Keymap Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/keymap.toml.md))
- `theme.toml` for theme configuration (see [Theme Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/theme.toml.md))
- `logger.toml` for logger configuration (see [Logger Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/logger.toml.md))
- `telegram.toml` for Telegram configuration (see [Telegram Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/telegram.toml.md))
