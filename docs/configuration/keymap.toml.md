# keymap.toml

## Default keymap configuration

```toml
# The core_window key bindings are usable in any app component.
[core_window]
keymap = [
  # Quit the application
  # Note that when the prompt is focused, the "q" key will be used to type the letter "q".
  { keys = ["q"], command = "try_quit", description = "Quit the application"},
  # Quit the application
  { keys = ["ctrl+c"], command = "try_quit", description = "Quit the application"},
  # Focus the chat list
  { keys = ["alt+1"], command = "focus_chat_list", description = "Focus the chat list"},
  { keys = ["alt+left"], command = "focus_chat_list", description = "Focus the chat list"},
  # Focus the chat
  { keys = ["alt+2"], command = "focus_chat", description = "Focus the chat"},
  { keys = ["alt+right"], command = "focus_chat", description = "Focus the chat"},
  # Focus the prompt
  { keys = ["alt+3"], command = "focus_prompt", description = "Focus the prompt"},
  { keys = ["alt+down"], command = "focus_prompt", description = "Focus the prompt"},
  # Unfocus the current component
  { keys = ["esc"], command = "unfocus_component", description = "Unfocus the current component"},
  { keys = ["alt+up"], command = "unfocus_component", description = "Unfocus the current component"},
  # Toggle chat_list visibility
  { keys = ["alt+n"], command = "toggle_chat_list", description = "Toggle chat_list visibility"},
  # Increase the chat list size
  { keys = ["alt+l"], command = "increase_chat_list_size", description = "Increase the chat list size"},
  # Decrease the chat list size
  { keys = ["alt+h"], command = "decrease_chat_list_size", description = "Decrease the chat list size"},
  # Increase the prompt size
  { keys = ["alt+k"], command = "increase_prompt_size", description = "Increase the prompt size"},
  # Decrease the prompt size
  { keys = ["alt+j"], command = "decrease_prompt_size", description = "Decrease the prompt size"},
]

# The chat_list key bindings are only usable in the chat list component.
# When the chat list is focused, the chat list key bindings will be active.
[chat_list]
keymap = [
  # Select the next chat
  { keys = ["down"], command = "chat_list_next", description = "Select the next chat"},
  # Select the previous chat
  { keys = ["up"], command = "chat_list_previous", description = "Select the previous chat"},
  # Unselect the current chat
  { keys = ["left"], command = "chat_list_unselect", description = "Unselect the current chat"},
  # Open the selected chat
  { keys = ["right"], command = "chat_list_open", description = "Open the selected chat"},
  # Open the selected chat
  { keys = ["enter"], command = "chat_list_open", description = "Open the selected chat"},
]

# The chat key bindings are only usable in the chat component.
# When the chat is focused, the chat key bindings will be active.
[chat]
keymap = [
  # Select the next message
  { keys = ["down"], command = "chat_window_next", description = "Select the next message"},
  # Select the previous message
  { keys = ["up"], command = "chat_window_previous", description = "Select the previous message"},
  # Unselect the current message
  { keys = ["left"], command = "chat_window_unselect", description = "Unselect the current message"},
  # Delete the selected message for all users
  { keys = ["d"], command = "chat_window_delete_for_everyone", description = "Delete the selected message for all users"},
  # Delete the selected message for "me"
  { keys = ["D"], command = "chat_window_delete_for_me", description = "Delete the selected message for 'me'"},
  # Copy the selected message
  { keys = ["y"], command = "chat_window_copy", description = "Copy the selected message"},
  # Copy the selected message
  { keys = ["ctrl+c"], command = "chat_window_copy", description = "Copy the selected message"},
  # Edit the selected message
  { keys = ["e"], command = "chat_window_edit", description = "Edit the selected message"},
  # Reply to the selected message
  { keys = ["r"], command = "chat_window_reply", description = "Reply to the selected message"},
]

# The prompt key bindings are only usable in the prompt component.
# When the prompt is focused, the prompt key bindings will be active.
[prompt]

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

## Example of a custom keymap configuration

Example of `keymap.toml`:

```toml
[core_window]
keymap = [
  # Quit the application with "q" followed by "a"
  { keys = ["q", "a"], command = "quit", description = "Quit the application with 'q' followed by 'a'"},
]
```
