# tgt.toml

# Default application configuration

```toml
# `mouse_support` enables mouse support in the terminal.
mouse_support = true
# `paste_support` enables paste support in the terminal.
paste_support = true
# `frame_rate` is the frame rate of the terminal in frames per second.
# The suggested frame rate are:
# - 120.0 for ultra smooth animations
# - 60.0 for smooth animations
# - 30.0 for normal animations
# - 15.0 for slow animations
frame_rate = 60.0
# `show_status_bar` enables the status bar at the bottom of the terminal.
show_status_bar = true
# `show_title_bar` enables the title bar at the top of the terminal.
show_title_bar = true
```

# Example of a custom application configuration

This is an example of a custom application configuration. This configuration will be merged with the default configuration.
It means that the default configuration will be overwritten by the custom configuration.

```toml
show_status_bar = false
show_title_bar = false
```
