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

/// A trait for configuration files.
/// This trait is used to define a configuration file and its associated configuration struct.
/// The configuration struct must implement the `Default` trait and the `Into` trait for the raw configuration type.
pub trait ConfigFile: Sized + Default {
        /// The raw configuration type.
        /// This type is used to parse the configuration file and must implement the `DeserializeOwned` trait.
        type Raw: Into<Self> + DeserializeOwned;
        /// Get the configuration type.
        ///
        /// # Returns
        /// The configuration type.
        fn get_type() -> ConfigType;
        /// Get the configuration file.
        ///
        /// # Returns
        /// The configuration file.
        fn get_config() -> Self {
                parse_config_or_default::<Self::Raw, Self>(Self::get_type().as_default_filename().as_str())
        }
}
/// Search the configuration directories for a file.
/// This function searches the configuration directories for the specified file name and returns the first match.
///
/// # Arguments
/// * `file_name` - The name of the file (including the file extension) to search for in the configuration directories.
///
/// # Returns
/// The path to the first matching file or `None` if no matching file is found.
pub fn search_config_directories(file_name: &str) -> Option<PathBuf> {
        CONFIG_DIR_HIERARCHY
                .iter()
                .map(|path| path.join(file_name))
                .find(|path| path.exists())
}
/// Parse a configuration file into a configuration struct.
/// This function attempts to parse the specified file and returns the parsed configuration.
///
/// # Arguments
/// * `file_path` - The path to the file to parse.
///
/// # Returns
/// The parsed configuration or an error if the file cannot be parsed.
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
/// Parse a configuration file or return the default configuration.
/// This function searches the configuration directories for the specified file name and attempts to parse it.
///
/// # Arguments
/// * `file_name` - The name of the file (including the file extension) to search for in the configuration directories.
///
/// # Returns
/// The parsed configuration or the default configuration if the file is not found or cannot be parsed.
pub fn parse_config_or_default<T, S>(file_name: &str) -> S
where
        T: DeserializeOwned + Into<S>,
        S: std::default::Default,
{
        // [TODO] Handle CLI arguments
        match search_config_directories(file_name) {
                Some(file_path) => match parse_file_to_config::<T, S>(&file_path) {
                        Ok(s) => {
                                tracing::info!("Loaded config from {}", file_path.display());
                                s
                        }
                        Err(e) => {
                                tracing::error!("Failed to parse {}: {}", file_name, e);
                                S::default()
                        }
                },
                None => {
                        tracing::warn!("No config file found for {}", file_name);
                        S::default()
                }
        }
}
