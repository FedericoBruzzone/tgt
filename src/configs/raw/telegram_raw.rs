use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
/// The telegram raw configuration.
pub struct TelegramRaw {
    /// The API ID.
    /// Note that the this field is used only if the `take_api_id_from_telegram_config` is `true`
    /// in the application configuration (`app.toml`).
    pub api_id: Option<String>,
    /// The API hash.
    /// Note that the this field is used only if the `take_api_hash_from_telegram_config` is `true`
    /// in the application configuration (`app.toml`).
    pub api_hash: Option<String>,
    /// The directory where the database is stored.
    pub database_dir: Option<String>,
    /// A flag that indicates if the user database should be used.
    pub use_file_database: Option<bool>,
    /// A flag that indicates if the chat info database should be used.
    pub use_chat_info_database: Option<bool>,
    /// A flag that indicates if the message database should be used.
    pub use_message_database: Option<bool>,
    /// A language code.
    pub system_language_code: Option<String>,
    /// The model of the device.
    pub device_model: Option<String>,
    /// A flag that indicates if the original file names should be ignored.
    pub ignore_file_names: Option<bool>,
}
