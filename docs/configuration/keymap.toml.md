# keymap.toml

## Default keymap configuration

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
  # Focus the chat list
  { keys = ["alt+1"], command = "focus_chat_list", description = "Focus the chat list"},
  # Focus the chat
  { keys = ["alt+2"], command = "focus_chat", description = "Focus the chat"},
  # Focus the prompt
  { keys = ["alt+3"], command = "focus_prompt", description = "Focus the prompt"},
  # Unfocus the current component
  { keys = ["esc"], command = "unfocus_component", description = "Unfocus the current component"},
  # Increase the chat list size
  { keys = ["shift+right"], command = "increase_chat_list_size", description = "Increase the chat list size"},
  # Decrease the chat list size
  { keys = ["shift+left"], command = "decrease_chat_list_size", description = "Decrease the chat list size"},
  # Increase the prompt size
  { keys = ["shift+up"], command = "increase_prompt_size", description = "Increase the prompt size"},
  # Decrease the prompt size
  { keys = ["shift+down"], command = "decrease_prompt_size", description = "Decrease the prompt size"},

]

# The chat_list key bindings are only usable in the chat list component.
# When the chat list is focused, the chat list key bindings will be active.
[chat_list]

# The chat key bindings are only usable in the chat component.
# When the chat is focused, the chat key bindings will be active.
[chat]

# The prompt key bindings are only usable in the prompt component.
# When the prompt is focused, the prompt key bindings will be active.
[prompt]

```

## Example of a custom logger configuration

This is an example of a custom keymap configuration. This configuration will be merged with the default configuration.
It means that the default key bindings will be overwritten by the custom key bindings.

```toml
[default]
keymap = [
  { keys = ["q"], command = "render", description = "Force a render"},
]
```
