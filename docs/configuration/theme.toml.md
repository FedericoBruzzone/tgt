# theme.toml

In the `theme.toml` you can define the `palette` (see palette section in the example below) and the styles for each component:

- `common`: In the common section you can define the styles that are common to all components. For example, the style for the focused border component.
- `chat_list`: In the chat_list section you can define the styles for the chat list component.
- `chat`: In the chat section you can define the styles for the chat component.
- `prompt`: In the prompt section you can define the styles for the prompt component.
- `status_bar`: In the status_bar section you can define the styles for the status bar component.
- `title_bar`: In the title_bar section you can define the styles for the title bar component.

Each component has a `self` style that defines the style of the component itself. The other styles are specific to the component and define the style of the elements inside the component.

## The Palette

The palette is a section in the theme configuration where you can define the colors that will be used in the theme. The colors defined in the palette can be used in the styles of the components. The palette section is optional, you can define the colors directly in the styles of the components but it is not recommended.

By default, each component of `tgt` use the colors defined in the palette. So, if you want to create a custom theme, you can change the colors in the palette and the components will use the new colors.

## Color Format

The supported color formats are:

- Hexadecimal: `#RGB` or `#RRGGBB` where `R`, `G`, and `B` are hexadecimal digits (not case-sensitive).
- RGB: `R, G, B` where `R`, `G`, and `B` are integers between 0 and 255.
- Palette: The palette colors are defined in the `palette` section of the theme configuration. For example, `primary`, `secondary`, `background`, etc.
- Default:
  - `black`: The default black color.
  - `red`: The default red color.
  - `green`: The default green color.
  - `yellow`: The default yellow color.
  - `blue`: The default blue color.
  - `magenta`: The default magenta color.
  - `cyan`: The default cyan color.
  - `gray`: The default gray color.
  - `dark_gray`: The default dark gray color.
  - `light_red`: The default light red color.
  - `light_green`: The default light green color.
  - `light_yellow`: The default light yellow color.
  - `light_blue`: The default light blue color.
  - `light_magenta`: The default light magenta color.
  - `light_cyan`: The default light cyan color.
  - `white`: The default white color.
  - `reset`: The default reset color.

## Default theme configuration

```toml
[palette]
black = "#000000"
white = "#ffffff"
background = "#000000"
primary = "#00548e"
primary_variant = "#0073b0"
primary_light = "#94dbf7"
secondary = "#ca3f04"
secondary_variant = "#e06819"
secondary_light = "#fcac77"
ternary = "#696969"
ternary_variant = "#808080"
ternary_light = "#6e7e85"
surface = "#141414"
on_surface = "#dcdcdc"
error = "#D50000"
on_error = "#FFCDD2"

[common]
border_component_focused = { fg = "secondary", bg = "background", bold = false, underline = false, italic = false }
item_selected = { fg = "", bg = "surface", bold = true, underline = false, italic = false }
timestamp = { fg = "ternary_light", bg = "background", bold = false, underline = false, italic = false }

[chat_list]
self = { fg = "primary", bg = "background", bold = false, underline = false, italic = false }
item_selected = { fg = "", bg = "primary", bold = false, underline = false, italic = false }
item_chat_name = { fg = "primary_light", bg = "background", bold = true, underline = false, italic = false }
item_message_content = { fg = "secondary_light", bg = "background", bold = false, underline = false, italic = true }
item_unread_counter = { fg = "secondary", bg = "background", bold = true, underline = false, italic = false }

[chat]
self = { fg = "primary", bg = "background", bold = false, underline = false, italic = false }
chat_name = { fg = "secondary", bg = "background", bold = true, underline = false, italic = false }
message_myself_name = { fg = "primary_light", bg = "background", bold = true, underline = false, italic = false }
message_myself_content = { fg = "primary_variant", bg = "background", bold = false, underline = false, italic = false }
message_other_name = { fg = "secondary_light", bg = "background", bold = true, underline = false, italic = false }
message_other_content = { fg = "secondary_variant", bg = "background", bold = false, underline = false, italic = false }
message_reply_text = { fg = "ternary", bg = "background", bold = false, underline = false, italic = false }
message_reply_name = { fg = "secondary_light", bg = "background", bold = true, underline = false, italic = false }
message_reply_content = { fg = "secondary_variant", bg = "background", bold = false, underline = false, italic = false }

[prompt]
self = { fg = "primary", bg = "background", bold = false, underline = false, italic = false }
message_text = { fg = "primary_light", bg = "background", bold = false, underline = false, italic = false }
message_text_selected = { fg = "secondary_light", bg = "ternary", bold = false, underline = false, italic = true }
message_preview_text = { fg = "ternary", bg = "background", bold = false, underline = false, italic = false }

[reply_message]
self = { fg = "secondary_light", bg = "background", bold = false, underline = false, italic = false }
message_text = { fg = "secondary_variant", bg = "background", bold = false, underline = false, italic = false }

[status_bar]
self = { fg = "on_surface", bg = "surface", bold = false, underline = false, italic = false }
size_info_text = { fg = "primary_light", bg = "surface", bold = false, underline = false, italic = false }
size_info_numbers = { fg = "secondary_light", bg = "surface", bold = false, underline = false, italic = true }
press_key_text = { fg = "primary_light", bg = "surface", bold = false, underline = false, italic = false }
press_key_key = { fg = "secondary_light", bg = "surface", bold = false, underline = false, italic = true }
message_quit_text = { fg = "primary_light", bg = "surface", bold = false, underline = false, italic = false }
message_quit_key = { fg = "secondary_light", bg = "surface", bold = false, underline = false, italic = true }
open_chat_text = { fg = "primary_light", bg = "surface", bold = false, underline = false, italic = false }
open_chat_name = { fg = "secondary_light", bg = "surface", bold = false, underline = false, italic = true }

[title_bar]
self = { fg = "on_surface", bg = "surface", bold = false, underline = false, italic = false }
title1 = { fg = "primary_light", bg = "surface", bold = true, underline = true, italic = true }
title2 = { fg = "secondary_light", bg = "surface", bold = true, underline = true, italic = true }
title3 = { fg = "ternary_light", bg = "surface", bold = true, underline = true, italic = false }
```

## Theme Discovery and Switching

`tgt` automatically discovers available themes by scanning for `.toml` files in the `themes/` subdirectory of your configuration directories. Themes are discovered dynamically at runtime, so you can add new theme files without restarting the application.

### Theme File Locations

Theme files should be placed in the `themes/` subdirectory within your configuration directory:

- **Default location**: `~/.tgt/config/themes/` (Linux/macOS) or `C:\Users\<name>\.tgt\config\themes\` (Windows)
- **Custom location**: `~/.config/tgt/config/themes/` (Linux/macOS) or `C:\Users\<name>\AppData\Roaming\tgt\config\themes\` (Windows)
- **Environment variable**: `$TGT_CONFIG_DIR/themes/` (if `TGT_CONFIG_DIR` is set)

### Theme Discovery Order

Themes are discovered in the following order (first match wins):

1. `$TGT_CONFIG_DIR/themes/` (if `TGT_CONFIG_DIR` environment variable is set)
2. `./config/themes/` (debug mode only, for development)
3. `~/.config/tgt/config/themes/` (if exists)
4. `~/.tgt/config/themes/` (if exists)

### Theme Naming

- Theme files must have a `.toml` extension
- The theme name is derived from the filename (without the `.toml` extension)
- For example, `monokai.toml` becomes the theme name `monokai`
- Themes are sorted alphabetically, with `theme` (the default) always appearing first

### Switching Themes

You can switch themes in two ways:

1. **Cycle through themes**: Use the theme switching keybinding (default: see keymap configuration) to cycle through all available themes
2. **Select specific theme**: Use the theme selector popup (default: see keymap configuration) to choose a specific theme from the list

When you switch themes, the selected theme is saved to your `app.toml` configuration file so it persists across restarts.

## Custom configuration

### How create a custom configuration file

`tgt` by default reads its **default** configurations from:
- Linux: `/home/<name>/.tgt/config/`
- macOS: `/Users/<name>/.tgt/config/`
- Windows: `C:\Users\<name>\.tgt\config/`

We suggest you to not modify this file, but to create your own **custom** configuration file in the following directories (in order of precedence):

- `$TGT_CONFIG_DIR` (if set)
- `$HOME/.config/tgt/config` (for Linux and macOS) and `C:\Users\<name>\AppData\Roaming\tgt\config` (for Windows)

Reading configurations from the following directories will override the fields defined in the default configuration files.
It means that the fields that are not present in the custom configuration will be taken from the default configuration, while the fields that are present in the custom configuration will override the default configuration.
Note that after the finding the first configuration file, `tgt` stops looking for more configurations, it is short-circuited.

### Creating Custom Themes

To create a custom theme:

1. Create a new `.toml` file in the `themes/` subdirectory of your configuration directory
2. Name it something descriptive (e.g., `my_theme.toml`)
3. Copy the structure from an existing theme file (like `theme.toml`) and modify the colors and styles
4. The theme will be automatically discovered and available for selection

**Example**: To create a custom theme called `dark_blue`:

1. Create `~/.config/tgt/config/themes/dark_blue.toml` (or in your preferred config directory)
2. Add your theme configuration following the structure shown in the default theme example above
3. The theme will appear in the theme selector automatically
4. Set `theme_filename = "themes/dark_blue.toml"` in your `app.toml` to make it the default

### Example of a custom theme configuration

Example of a custom theme file (e.g., `themes/my_custom_theme.toml`):

```toml
[palette]
test_color = "#ff0000"
background = "#ffffff"

[common]
border_component_focused = { fg = "test_color", bg = "background", bold = false, underline = false, italic = false }
```

**Note**: When creating custom themes, place them in the `themes/` subdirectory. The theme will be automatically discovered and can be selected via the theme switcher or by setting `theme_filename = "themes/my_custom_theme.toml"` in your `app.toml`.

