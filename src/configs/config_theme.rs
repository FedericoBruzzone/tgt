use {crate::THEME_CONFIG, ratatui::style::Style};

pub fn status_bar_size_info() -> Style {
    THEME_CONFIG.status_bar.get("size_info").unwrap().as_style()
}
