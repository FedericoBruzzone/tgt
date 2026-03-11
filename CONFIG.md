# Configuration

`tgt` is fully customizable. This document describes how configuration works, where files live, and how to manage them.

## Overview of config behavior

- **XDG paths**: Config now follows [XDG Base Directory](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) (e.g. `~/.config/tgt` on Linux/macOS, `%APPDATA%\tgt` on Windows), with **backwards compatibility**: if a legacy `~/.tgt` (or platform-equivalent) directory exists, it is still used so existing setups keep working.
- **Auto-creation**: Config (and data/log) directories are created at startup if missing, so you can run `tgt` without manual setup.
- **Versioning**: Configs are versioned. If the program detects a version mismatch, it corrects the configuration and adds any missing keybindings so upgrades stay consistent.
- **CLI**: You can generate an initial config or remove config/data/log folders for a clean slate:
  - **Generate initial config**: `tgt init-config` (only writes missing files; use `tgt init-config --force` to overwrite).
  - **Remove config/data/logs**: `tgt clear --config` / `--data` / `--logs`, or `tgt clear --all`. Use `--yes` to skip confirmation.
- **Bundled defaults**: Default configs are bundled in the binary, so `cargo install tgt` works regardless of install path. A fallback to bundled config is always available, so the application always has a default config to use.
- **Tests**: The config pipeline (paths, merge, versioning) is covered by tests to verify behavior.

## Config directory locations

- **Preferred (XDG)**  
  - Linux/macOS: `$XDG_CONFIG_HOME/tgt` (default `~/.config/tgt`).  
  - Windows: `%APPDATA%\tgt`.
- **Legacy (backwards compatibility)**  
  - If `~/.tgt` (or the equivalent on your OS) already exists, it is used instead of the XDG path.

Config files live under the **config** subdirectory (e.g. `~/.config/tgt/config/`). Data and logs use the same base (e.g. `~/.config/tgt/` for data/state).

## Configuration files

In the config directory, `tgt` uses:

| File | Purpose |
|------|--------|
| `app.toml` | General application settings (e.g. message status icons: ASCII vs emoji). See [app.toml.md](docs/configuration/app.toml.md). |
| `keymap.toml` | Keybindings. See [keymap.toml.md](docs/configuration/keymap.toml.md). |
| `logger.toml` | Logging. See [logger.toml.md](docs/configuration/logger.toml.md). |
| `telegram.toml` | Telegram API and TDLib. See [telegram.toml.md](docs/configuration/telegram.toml.md). |
| `theme.toml` | Color theme. See [theme.toml.md](docs/configuration/theme.toml.md). |

Theme files can also live in a `themes/` subdirectory; multiple `.toml` themes are discovered there. See [docs/configuration/README.md](docs/configuration/README.md) for theme layout and switching.

> **Note**  
> The theme switcher only persists the selected theme in **release** mode. In debug, the chosen theme is not saved between sessions.

> **Warning**  
> End-users are expected to supply their own `api_id` and `api_hash` when possible. [telegram.toml.md](docs/configuration/telegram.toml.md) has more on this.

## Default keybindings

_None state_:

```bash
esc:               to the "None" state
alt+1 | alt+left:  Focus on the chat list
alt+2 | alt+right: Focus on the chat
alt+3 | alt+down:  Focus on the prompt
alt+h | alt+l:     Resize the chat list
alt+j | alt+k:     Resize the prompt
alt+n:             Toggle chat list
alt+r:             Start chat list search (when nothing is selected)
alt+c:             Restore the default ordering of the chat list
alt+f1:            Show command guide with all keybindings
alt+t:             Show theme selector
q | ctrl+c:        Quit
```

_Chat List_

```bash
up | down:     Move selection
enter | right: Open the chat
left:          Unselect chat
alt+r:         Focus on prompt to start searching
alt+c:         Restore the default ordering of the chat list

esc:               Return to the "None" state
alt+1 | alt+left:  Focus on the chat list
alt+2 | alt+right: Focus on the chat
alt+3 | alt+down:  Focus on the prompt
```

_Chat_

```bash
up | down: Scroll the messages
left:      Unselect message
y:         Copy the message
e:         Edit the message
r:         Reply to the message
d:         Delete the message for everyone
D:         Delete the message for me
alt+v:     View photo from selected message (opens photo viewer)
alt+r:     Focus on prompt to search messages in the chat window
alt+c:     Restore the default ordering of messages

esc:               Return to the "None" state
alt+1 | alt+left:  Focus on the chat list
alt+2 | alt+right: Focus on the chat
alt+3 | alt+down:  Focus on the prompt
```

_Photo Viewer_  
Open from the chat by selecting a photo message and pressing `alt+v`. When the photo viewer is focused:

```bash
esc :   Close the photo viewer
up | k:    View previous message
down | j:  View next message

esc:               Return to the "None" state
alt+1 | alt+left:  Focus on the chat list
alt+2 | alt+right: Focus on the chat
alt+3 | alt+down:  Focus on the prompt
```

_Prompt_

Note that when the prompt is focused, you can **NOT** use `q` or `ctrl+c` to quit the application, you need to press `esc` to return to the "None" state.

```bash
alt+enter:                        Send the message

left | right | up | down:         Move the cursor
ctrl+left | ctrl+b:               Move the cursor to the previous word
ctrl+right | ctrl+f:              Move the cursor to the next word
ctrl+alt+left | ctrl+a | home:    Move the cursor to the beginning of the line (also ctrl+left+b | shift+super+left | shift+super+b)
ctrl+alt+right | ctrl+e | end:    Move the cursor to the end of the line (also ctrl+right+f | shift+super+right | shift+super+f)

shift+left:                       Move the cursor left and select the text
shift+right:                      Move the cursor right and select the text
shift+up:                         Move the cursor up and select the text
shift+down:                        Move the cursor down and select the text
shift+ctrl+left:                  Select the text before the cursor
shift+ctrl+right:                 Select the text after the cursor

ctrl+c:                           Copy the selected text
ctrl+v:                           Paste the copied text

ctrl+w | ctrl+backspace | ctrl+h: Delete the word before the cursor

esc:               Return to the "None" state
alt+1 | alt+left:  Focus on the chat list
alt+2 | alt+right: Focus on the chat
alt+3 | alt+down:  Focus on the prompt
```

_Mouse_

```bash
Scroll:   In chat list or chat to move selection or messages
Chat list: First click focuses the list, second click on an item opens that chat
```
