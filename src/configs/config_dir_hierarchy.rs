use {
  crate::{
    app_error::AppError,
    configs::{config_type::ConfigType, TGT_CONFIG_HOME, TGT_PROGRAM_NAME},
  },
  config::{Config, File, FileFormat},
  lazy_static::lazy_static,
  serde::de::DeserializeOwned,
  std::path::{Path, PathBuf},
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

    if let Some(p) = if cfg!(target_os = "macos") {
      dirs::home_dir().map(|h| h.join(".config"))
    } else {
      dirs::config_dir()
    } {
      let mut p = p;
      p.push(TGT_PROGRAM_NAME);
      if p.is_dir() {
        config_dirs.push(p);
      }
    }

    config_dirs
  };
}

pub trait ConfigFile: Sized + Default {
  type Raw: Into<Self> + DeserializeOwned;

  fn get_type() -> ConfigType;

  fn get_config() -> Self {
    parse_config_or_default::<Self::Raw, Self>(Self::get_type().as_filename())
  }
}

pub fn search_config_directories(file_name: &str) -> Option<PathBuf> {
  search_directories(file_name, &CONFIG_DIR_HIERARCHY)
}

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
  let builder: T = Config::builder()
    .add_source(File::from(file_path).format(FileFormat::Toml))
    .build()?
    .try_deserialize::<T>()?;
  Ok(builder.into())
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
