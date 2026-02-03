pub const SMALL_AREA_WIDTH: u16 = 100;
pub const SMALL_AREA_HEIGHT: u16 = 20;
pub const MAX_CHAT_LIST_SIZE: u16 = 25;
pub const MIN_CHAT_LIST_SIZE: u16 = 10;
pub const MAX_PROMPT_SIZE: u16 = 20;
pub const MIN_PROMPT_SIZE: u16 = 3;

pub mod chat_list_window;
pub mod chat_window;
pub mod command_guide;
pub mod component_traits;
pub mod core_window;
pub mod photo_viewer;
pub mod prompt_window;
pub mod reply_message;
pub mod search_overlay;
#[cfg(test)]
pub mod search_tests;
pub mod status_bar;
pub mod theme_selector;
pub mod title_bar;
