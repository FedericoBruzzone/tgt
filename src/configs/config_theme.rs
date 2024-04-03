use {
    crate::{APP_CONFIG, THEME_CONFIG},
    ratatui::style::Style,
};

// Marco for decorate the style if theme_enable is true
macro_rules! theme_style {
    ($style: expr) => {
        if APP_CONFIG.theme_enable {
            $style.as_style()
        } else {
            Style::default()
        }
    };
}

pub fn status_bar_size_info() -> Style {
    theme_style!(THEME_CONFIG.status_bar.get("size_info").unwrap())
}
