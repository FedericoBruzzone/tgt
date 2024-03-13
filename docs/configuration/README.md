# Configuration

`tgt` by default reads its default configurations from the `config` directory in the root of the project. After this, it reads configurations from the following directories (in order of precedence) and overwrites the fields specified in the default configuration:

- `$TGT_CONFIG_HOME` (if set)
- `$HOME/.config/tgt` (for Linux and macOS) and `C:\Users\<name>\AppData\Roaming\tgt` (for Windows)

Note that after the finding the first configuration file, `tgt` stops looking for more configurations, it is short-circuited.

## Configuration Files

In the configuration directory, `tgt` looks for the following files:
- `logger.toml` for logger configuration (see [Logger Configuration](https://github.com/FedericoBruzzone/tgt/blob/main/docs/configuration/logger.toml.md))
