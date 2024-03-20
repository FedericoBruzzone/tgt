# keymap.toml

# Default keymap configuration

```toml
# The default key bindings are usable in any app component.
[default]
keymap = [
  # Quit the application
  { keys = ["q"], command = "quit", description = "Quit the application"},
  # Quit the application
  { keys = ["ctrl+c"], command = "quit", description = "Quit the application"},
  # Quit the application
  { keys = ["w", "w"], command = "quit", description = "Quit the application"},
]

# The chat_list key bindings are only usable in the chat list component.
# When the chat list is focused, the chat list key bindings will be active.
[chats_list]

# The chat key bindings are only usable in the chat component.
# When the chat is focused, the chat key bindings will be active.
[chat]

# The prompt key bindings are only usable in the prompt component.
# When the prompt is focused, the prompt key bindings will be active.
[prompt]
```

# Example of a custom logger configuration

This is an example of a custom keymap configuration. This configuration will be merged with the default configuration.
It means that the default key bindings will be overwritten by the custom key bindings.

```toml
[default]
keymap = [
  { keys = ["q"], command = "render", description = "Force a render"},
]
```
