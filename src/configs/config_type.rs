use {
        config::FileFormat,
        std::fmt::{Display, Formatter, Result},
};

#[derive(Copy, Clone, Debug)]
/// `ConfigType` is an enum that represents the different types of configuration files that the application can use.
/// The different types of configuration files are:
/// * App - The application configuration file.
/// * Keymap - The keymap configuration file.
/// * Logger - The logger configuration file.
/// * Theme - The theme configuration file.
pub enum ConfigType {
        App,
        Keymap,
        Logger,
        Theme,
}
/// Implement the `ConfigType` enum.
impl ConfigType {
        /// Get the different types of configuration files that the application can use.
        pub const fn enumerate() -> &'static [Self] {
                &[Self::App, Self::Keymap, Self::Logger, Self::Theme]
        }
        /// Get the file name without the file extension for the configuration file type.
        ///
        /// # Returns
        /// * `&'static str` - The file name without the file extension.
        pub const fn as_str(&self) -> &'static str {
                match self {
                        Self::App => "tgt",
                        Self::Keymap => "keymap",
                        Self::Logger => "logger",
                        Self::Theme => "theme",
                }
        }
        /// Get the default file extension for the configuration file type.
        /// The default file extension is `.toml`.
        ///
        /// # Returns
        /// * `&'static str` - The default file extension.
        const fn default_format(&self) -> &'static str {
                match self {
                        Self::App => ".toml",
                        Self::Keymap => ".toml",
                        Self::Logger => ".toml",
                        Self::Theme => ".toml",
                }
        }
        /// Get the default file name for the configuration file type.
        ///
        /// # Returns
        /// * `String` - The default file name.
        pub fn as_default_filename(&self) -> String {
                format!("{}{}", self.as_str(), self.default_format())
        }
        pub const fn supported_formats(&self) -> &'static [FileFormat] {
                let formats = self.get_supported_formats();
                match self {
                        Self::App => formats,
                        Self::Keymap => formats,
                        Self::Logger => formats,
                        Self::Theme => formats,
                }
        }
        /// Get the supported file formats for the configuration file type.
        /// The supported file formats are:
        /// * Json5
        /// * Json
        /// * Yaml
        /// * Toml
        /// * Ini
        /// * Ron
        ///
        /// # Returns
        /// * `&'static [FileFormat]` - The supported file formats.
        const fn get_supported_formats(&self) -> &'static [FileFormat] {
                &[
                        FileFormat::Json5,
                        FileFormat::Json,
                        FileFormat::Yaml,
                        FileFormat::Toml,
                        FileFormat::Ini,
                        FileFormat::Ron,
                ]
        }
}

/// Implement the `Display` trait for `ConfigType`.
impl Display for ConfigType {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                f.write_str(self.as_str())
        }
}
