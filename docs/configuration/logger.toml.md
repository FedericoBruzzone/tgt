# logger.toml

## Default logger configuration

```toml
# `log_dir` is the folder where the log file will be created.
# This folder is relative to the `tgt` home directory.
log_dir = ".data/logs"
# `log_file` is the name of the log file.
log_file = "tgt.log"
# The rotation frequency of the log.
# The log rotation frequency can be one of the following:
# - minutely: A new log file in the format of log_dir/log_file.yyyy-MM-dd-HH-mm will be created minutely (once per minute)
# - hourly: A new log file in the format of log_dir/log_file.yyyy-MM-dd-HH will be created hourly
# - daily: A new log file in the format of log_dir/log_file.yyyy-MM-dd will be created daily
# - never: This will result in log file located at log_dir/log_file
rotation_frequency = "daily"
# The maximum number of old log files that will be stored
max_old_log_files = 7
# `log_level` is the level of logging.
# The levels are (based on `RUST_LOG`):
# - error: only log errors
# - warn: log errors and warnings
# - info: log errors, warnings and info
# - debug: log errors, warnings, info and debug
# - trace: log errors, warnings, info, debug and trace
# - off: turn off logging
log_level = "info"
```

## Custom logger configuration

### How create a custom configuration file

`tgt` by default reads its **default** configurations from:
- Linux: `/home/<name>/.tgt/config/`
- macOS: `/Users/<name>/.tgt/config/`
- Windows: `C:\Users\<name>\.tgt\config/`

We suggest you to not modify these files, but to create your own **custom** configuration files in the following directories (in order of precedence):

- `$TGT_CONFIG_DIR` (if set)
- `$HOME/.config/tgt/` (for Linux and macOS) and `C:\Users\<name>\AppData\Roaming\tgt\` (for Windows)

Reading configurations from the following directories will override the fields defined in the default configuration files.
It means that the fields that are not present in the custom configuration will be taken from the default configuration, while the fields that are present in the custom configuration will override the default configuration.
Note that after the finding the first configuration file, `tgt` stops looking for more configurations, it is short-circuited.

### Example of a custom logger configuration

Example of `logger.toml`:

```toml
log_level = "off"
```
