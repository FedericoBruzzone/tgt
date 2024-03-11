use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct LoggerRaw {
  pub log_folder: String,
  pub log_file: String,
  pub log_level: String,
}
