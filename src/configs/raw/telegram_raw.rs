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
}
