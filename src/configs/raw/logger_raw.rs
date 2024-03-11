use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
/// The raw logger configuration.
pub struct LoggerRaw {
        pub log_folder: Option<String>,
        pub log_file: Option<String>,
        pub log_level: Option<String>,
}
