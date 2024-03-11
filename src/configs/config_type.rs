use config::FileFormat;

#[derive(Copy, Clone, Debug)]
pub enum ConfigType {
  App,
  Keymap,
  Logger,
  Theme,
}

impl std::fmt::Display for ConfigType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl ConfigType {
  pub const fn enumerate() -> &'static [Self] {
    &[Self::App, Self::Keymap, Self::Logger, Self::Theme]
  }

  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::App => "tgt",
      Self::Keymap => "keymap",
      Self::Logger => "logger",
      Self::Theme => "theme",
    }
  }

  pub const fn as_filename(&self) -> &'static str {
    match self {
      Self::App => "tgt",
      Self::Keymap => "keymap",
      Self::Logger => "logger",
      Self::Theme => "theme",
    }
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
