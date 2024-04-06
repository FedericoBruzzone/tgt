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

# Default theme configuration

```toml
[palette]
black = "#000000"
white = "#ffffff"
primary = "#00548e"
primary_variant = "#0073b0"
secondary = "#ca2504"
secondary_variant = "#e33610"
background = "#000000"
surface = "#000000"
error = "#D50000"
on_primary = "#b2e3f7"
on_secondary = "#ffcbbb"
on_background = "#ffffff"
on_surface = "#ffffff"
on_error = "#FFCDD2"
# ternary = "#efba5d"
# on_ternary = "#f47868"

[common]
border_component_focused = { fg = "secondary_variant", bg = "background", bold = false, underline = false, italic = false }
item_selected = { fg = "on_secondary", bg = "secondary", bold = false, underline = false, italic = true }

[chat_list]
self = { fg = "on_primary", bg = "background", bold = false, underline = false, italic = false }

[chat]
self = { fg = "on_primary", bg = "background", bold = false, underline = false, italic = false }

[prompt]
self = { fg = "on_primary", bg = "background", bold = false, underline = false, italic = false }

[status_bar]
self = { fg = "on_surface", bg = "surface", bold = false, underline = false, italic = false }
size_info_text = { fg = "on_primary", bg = "surface", bold = false, underline = false, italic = false }
size_info_numbers = { fg = "on_secondary", bg = "surface", bold = false, underline = false, italic = true }
press_key_text = { fg = "on_primary", bg = "surface", bold = false, underline = false, italic = false }
press_key_key = { fg = "on_secondary", bg = "surface", bold = false, underline = false, italic = true }
message_quit_text = { fg = "on_primary", bg = "surface", bold = false, underline = false, italic = false }
message_quit_key = { fg = "on_secondary", bg = "surface", bold = false, underline = false, italic = true }

[title_bar]
self = { fg = "on_surface", bg = "surface", bold = false, underline = false, italic = false }
```

