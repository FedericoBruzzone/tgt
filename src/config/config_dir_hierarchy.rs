use {
  crate::{
    app_error::AppError,
    config::{TGT_CONFIG_HOME, TGT_PROGRAM_NAME},
  },
  lazy_static::lazy_static,
  serde::de::DeserializeOwned,
  std::{
    fs,
    path::{Path, PathBuf},
  },
};

lazy_static! {
  static ref CONFIG_DIR_HIERARCHY: Vec<PathBuf> = {
    let mut config_dirs = vec![];

    if let Ok(p) = std::env::var(TGT_CONFIG_HOME) {
      let p = PathBuf::from(p);
      if p.is_dir() {
        config_dirs.push(p);
      }
    }

    #[cfg(not(windows))]
    if let Ok(dirs) = xdg::BaseDirectories::with_prefix(TGT_PROGRAM_NAME) {
      config_dirs.push(dirs.get_config_home());
    }

    if let Ok(p) = std::env::var("HOME") {
      let mut p = PathBuf::from(p);
      p.push(format!(".config/{}", TGT_PROGRAM_NAME));
      if p.is_dir() {
        config_dirs.push(p);
      }
    }

    config_dirs
  };
}

/// Search for a file in the `CONFIG_DIR_HIERARCHY` and return the first match found.
/// If no match is found, return None.
/// By default, the `CONFIG_DIR_HIERARCHY` is set to the following directories:
/// - The value of the `TGT_CONFIG_HOME` environment variable, if set.
/// - The value of the `XDG_CONFIG_HOME` environment variable, if set.
/// - `$HOME/.config/tgt_program_name`, if the `HOME` environment variable is set.
///
/// # Arguments
/// * `file_name` - The name of the file to search for.
///
/// # Returns
/// The path to the first match found, or None if no match is found.
pub fn search_config_directories(file_name: &str) -> Option<PathBuf> {
  search_directories(file_name, &CONFIG_DIR_HIERARCHY)
}

/// Search for a file in a list of directories and return the first match found.
/// If no match is found, return None.
///
/// # Arguments
/// * `file_name` - The name of the file to search for.
/// * `directories` - A list of directories to search in.
///
/// # Returns
/// The path to the first match found, or None if no match is found.
pub fn search_directories<P>(file_name: &str, directories: &[P]) -> Option<PathBuf>
where
  P: AsRef<Path>,
{
  directories
    .iter()
    .map(|path| path.as_ref().join(file_name))
    .find(|path| path.exists())
}

fn parse_file_to_config<T, S>(file_path: &Path) -> Result<S, AppError>
where
  T: DeserializeOwned + Into<S>,
{
  let file_contents = fs::read_to_string(file_path)?;
  let config = toml::from_str::<T>(&file_contents)?;
  Ok(config.into())
}

pub fn parse_config_or_default<T, S>(file_name: &str) -> S
where
  T: DeserializeOwned + Into<S>,
  S: std::default::Default,
{
  match search_config_directories(file_name) {
    Some(file_path) => match parse_file_to_config::<T, S>(&file_path) {
      Ok(s) => s,
      Err(e) => {
        eprintln!("Failed to parse {}: {}", file_name, e);
        S::default()
      }
    },
    None => S::default(),
  }
}
