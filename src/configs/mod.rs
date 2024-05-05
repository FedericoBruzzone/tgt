use crate::app_error::AppError;
use config::Config;
use config::File;
use config::FileFormat;
use serde::de::DeserializeOwned;
use std::path::Path;

pub mod custom;
pub mod raw;

pub mod config_file;
pub mod config_theme;
pub mod config_type;

/// Deserialize a configuration file into a configuration struct.
/// This function attempts to parse the specified file and returns the parsed
/// configuration. If the file cannot be parsed, an error is returned.
///
/// # Arguments
/// * `file_path` - The path to the file to parse.
///
/// # Returns
/// The parsed configuration or an error if the file cannot be parsed.
pub fn deserialize_to_config<R>(file_path: &Path) -> Result<R, AppError<()>>
where
    R: DeserializeOwned,
{
    let builder: R = Config::builder()
        .add_source(File::from(file_path).format(FileFormat::Toml))
        .build()?
        .try_deserialize::<R>()?;
    Ok(builder)
}
/// Deserialize a configuration file into a configuration struct and convert it
/// into another configuration struct. This function attempts to parse the
/// specified file and returns the parsed configuration. If the file cannot be
/// parsed, an error is returned.
///
/// # Arguments
/// * `file_path` - The path to the file to parse.
///
/// # Returns
/// The parsed configuration or an error if the file cannot be parsed.
pub fn deserialize_to_config_into<R, S>(file_path: &Path) -> Result<S, AppError<()>>
where
    R: DeserializeOwned + Into<S>,
{
    deserialize_to_config::<R>(file_path).map(|s| s.into())
}
