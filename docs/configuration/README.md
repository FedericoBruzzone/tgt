# Configuration

`tgt` reads configurations from the following directories using environment variables (in order of precedence):

- `$TGT_CONFIG_HOME`
- `$XDG_CONFIG_HOME/tgt` (only on Linux and macOS)
- `$HOME/.config/tgt`

`tgt`'s behavior is:

If there exists a config file, use that config. (No default or inherited values from a default config)
If there is no config file, a default config will be used (found under config/)
