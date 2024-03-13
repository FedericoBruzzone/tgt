# logger.toml

# Default logger configuration

```toml
# `log_folder` is the folder where the log file will be created
log_folder = ".data"
# `log_file` is the name of the log file
log_file = "tgt.log"
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

# Example of a custom logger configuration

```toml
log_level = "debug"
```
