use crate::{
    app_error::AppError,
    configs::{
        self, config_file::ConfigFile, config_type::ConfigType, raw::telegram_raw::TelegramRaw,
    },
    utils,
};
use std::path::Path;

#[derive(Clone, Debug)]
/// The telegram configuration.
pub struct TelegramConfig {
    /// The API ID.
    /// Note that the this field is used only if the `take_api_id_from_telegram_config` is `true`
    /// in the application configuration (`app.toml`).
    pub api_id: String,
    /// The API hash.
    /// Note that the this field is used only if the `take_api_hash_from_telegram_config` is `true`
    /// in the application configuration (`app.toml`).
    pub api_hash: String,
    /// The directory where the database is stored.
    pub database_dir: String,
    /// A flag that indicates if the user database should be used.
    pub use_file_database: bool,
    /// A flag that indicates if the chat info database should be used.
    pub use_chat_info_database: bool,
    /// A flag that indicates if the message database should be used.
    pub use_message_database: bool,
    /// A language code.
    pub system_language_code: String,
    /// The model of the device.
    pub device_model: String,
    /// A flag that indicates if the original file names should be ignored.
    pub ignore_file_names: bool,
}
/// The telegram configuration implementation.
impl TelegramConfig {
    /// Get the default telegram configuration.
    ///
    /// # Returns
    /// The default telegram configuration.
    pub fn default_result() -> Result<Self, AppError<()>> {
        configs::deserialize_to_config_into::<TelegramRaw, Self>(Path::new(
            &configs::custom::default_config_telegram_file_path()?,
        ))
    }
}
/// The implementation of the configuration file for telegram.
impl ConfigFile for TelegramConfig {
    type Raw = TelegramRaw;

    fn get_type() -> ConfigType {
        ConfigType::Telegram
    }

    fn override_fields() -> bool {
        true
    }

    fn merge(&mut self, other: Option<Self::Raw>) -> Self {
        match other {
            None => self.clone(),
            Some(_other) => {
                tracing::info!("Merging telegram config");
                if let Some(api_id) = _other.api_id {
                    self.api_id = api_id;
                }
                if let Some(api_hash) = _other.api_hash {
                    self.api_hash = api_hash;
                }
                if let Some(database_dir) = _other.database_dir {
                    self.database_dir = database_dir;
                }
                if let Some(use_file_database) = _other.use_file_database {
                    self.use_file_database = use_file_database;
                }
                if let Some(use_chat_info_database) = _other.use_chat_info_database {
                    self.use_chat_info_database = use_chat_info_database;
                }
                if let Some(use_message_database) = _other.use_message_database {
                    self.use_message_database = use_message_database;
                }
                if let Some(system_language_code) = _other.system_language_code {
                    self.system_language_code = system_language_code;
                }
                if let Some(device_model) = _other.device_model {
                    self.device_model = device_model;
                }
                self.clone()
            }
        }
    }
}
/// The default telegram configuration.
impl Default for TelegramConfig {
    fn default() -> Self {
        Self::default_result().unwrap()
    }
}
/// The conversion from the raw telegram configuration to the telegram
/// configuration.
impl From<TelegramRaw> for TelegramConfig {
    fn from(raw: TelegramRaw) -> Self {
        Self {
            api_id: raw.api_id.unwrap(),
            api_hash: raw.api_hash.unwrap(),
            database_dir: utils::tgt_dir()
                .unwrap()
                .join(raw.database_dir.unwrap())
                .to_string_lossy()
                .to_string(),
            use_file_database: raw.use_file_database.unwrap(),
            use_chat_info_database: raw.use_chat_info_database.unwrap(),
            use_message_database: raw.use_message_database.unwrap(),
            system_language_code: raw.system_language_code.unwrap(),
            device_model: raw.device_model.unwrap(),
            ignore_file_names: raw.ignore_file_names.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        configs::{
            config_file::ConfigFile, custom::telegram_custom::TelegramConfig,
            raw::telegram_raw::TelegramRaw,
        },
        utils,
    };

    #[test]
    fn test_telegram_config_default() {
        let telegram_config = TelegramConfig::default();
        assert_eq!(telegram_config.api_id, "94575");
        assert_eq!(telegram_config.api_hash, "a3406de8d171bb422bb6ddf3bbd800e2");
    }

    #[test]
    fn test_telegram_from_raw() {
        let telegram_raw = TelegramRaw {
            api_id: Some("api_id".to_string()),
            api_hash: Some("api_hash".to_string()),
            database_dir: Some("database_dir".to_string()),
            use_file_database: Some(true),
            use_chat_info_database: Some(true),
            use_message_database: Some(true),
            system_language_code: Some("system_language_code".to_string()),
            device_model: Some("device_model".to_string()),
            ignore_file_names: Some(true),
        };
        let telegram_config = TelegramConfig::from(telegram_raw);
        assert_eq!(telegram_config.api_id, "api_id");
        assert_eq!(telegram_config.api_hash, "api_hash");
        assert_eq!(
            telegram_config.database_dir,
            utils::tgt_dir()
                .unwrap()
                .join("database_dir")
                .to_string_lossy()
                .to_string()
        );
        assert!(telegram_config.use_file_database);
        assert!(telegram_config.use_chat_info_database);
        assert!(telegram_config.use_message_database);
        assert_eq!(telegram_config.system_language_code, "system_language_code");
        assert_eq!(telegram_config.device_model, "device_model");
        assert!(telegram_config.ignore_file_names);
    }

    #[test]
    fn test_telegram_merge() {
        let mut telegram_config = TelegramConfig {
            api_id: "api_id".to_string(),
            api_hash: "api_hash".to_string(),
            database_dir: "database_dir".to_string(),
            use_file_database: false,
            use_chat_info_database: false,
            use_message_database: false,
            system_language_code: "system_language_code".to_string(),
            device_model: "device_model".to_string(),
            ignore_file_names: false,
        };
        let telegram_raw = TelegramRaw {
            api_id: Some("api_id_2".to_string()),
            api_hash: Some("api_hash_2".to_string()),
            database_dir: Some("database_dir_2".to_string()),
            use_file_database: Some(true),
            use_chat_info_database: Some(true),
            use_message_database: Some(true),
            system_language_code: Some("system_language_code_2".to_string()),
            device_model: Some("device_model_2".to_string()),
            ignore_file_names: Some(true),
        };
        let telegram_config = telegram_config.merge(Some(telegram_raw));
        assert_eq!(telegram_config.api_id, "api_id_2");
        assert_eq!(telegram_config.api_hash, "api_hash_2");
        assert_eq!(telegram_config.database_dir, "database_dir_2");
        assert!(telegram_config.use_file_database);
        assert!(telegram_config.use_chat_info_database);
        assert!(telegram_config.use_message_database);
        assert_eq!(
            telegram_config.system_language_code,
            "system_language_code_2"
        );
        assert_eq!(telegram_config.device_model, "device_model_2");
        assert!(!telegram_config.ignore_file_names);
    }

    #[test]
    fn test_telegram_merge_none() {
        let mut telegram_config = TelegramConfig {
            api_id: "api_id".to_string(),
            api_hash: "api_hash".to_string(),
            database_dir: "database_dir".to_string(),
            use_file_database: false,
            use_chat_info_database: false,
            use_message_database: false,
            system_language_code: "system_language_code".to_string(),
            device_model: "device_model".to_string(),
            ignore_file_names: false,
        };
        let telegram_config = telegram_config.merge(None);
        assert_eq!(telegram_config.api_id, "api_id");
        assert_eq!(telegram_config.api_hash, "api_hash");
        assert_eq!(telegram_config.database_dir, "database_dir");
        assert!(!telegram_config.use_file_database);
        assert!(!telegram_config.use_chat_info_database);
        assert!(!telegram_config.use_message_database);
        assert_eq!(telegram_config.system_language_code, "system_language_code");
        assert_eq!(telegram_config.device_model, "device_model");
        assert!(!telegram_config.ignore_file_names);
    }

    #[test]
    fn test_telegram_merge_partial() {
        let mut telegram_config = TelegramConfig {
            api_id: "api_id".to_string(),
            api_hash: "api_hash".to_string(),
            database_dir: "database_dir".to_string(),
            use_file_database: false,
            use_chat_info_database: false,
            use_message_database: false,
            system_language_code: "system_language_code".to_string(),
            device_model: "device_model".to_string(),
            ignore_file_names: false,
        };
        let telegram_raw = TelegramRaw {
            api_id: Some("api_id_2".to_string()),
            api_hash: None,
            database_dir: None,
            use_file_database: None,
            use_chat_info_database: None,
            use_message_database: None,
            system_language_code: None,
            device_model: None,
            ignore_file_names: None,
        };
        let telegram_config = telegram_config.merge(Some(telegram_raw));
        assert_eq!(telegram_config.api_id, "api_id_2");
        assert_eq!(telegram_config.api_hash, "api_hash");
        assert_eq!(telegram_config.database_dir, "database_dir");
        assert!(!telegram_config.use_file_database);
        assert!(!telegram_config.use_chat_info_database);
        assert!(!telegram_config.use_message_database);
        assert_eq!(telegram_config.system_language_code, "system_language_code");
        assert_eq!(telegram_config.device_model, "device_model");
        assert!(!telegram_config.ignore_file_names);
    }

    #[test]
    fn test_telegram_config_override_fields() {
        assert!(TelegramConfig::override_fields());
    }

    #[test]
    fn test_merge_all_fields() {
        let mut telegram_config = TelegramConfig {
            api_id: "api_id".to_string(),
            api_hash: "api_hash".to_string(),
            database_dir: "database_dir".to_string(),
            use_file_database: false,
            use_chat_info_database: false,
            use_message_database: false,
            system_language_code: "system_language_code".to_string(),
            device_model: "device_model".to_string(),
            ignore_file_names: false,
        };
        let telegram_raw = TelegramRaw {
            api_id: Some("api_id_2".to_string()),
            api_hash: Some("api_hash_2".to_string()),
            database_dir: Some("database_dir_2".to_string()),
            use_file_database: Some(true),
            use_chat_info_database: Some(true),
            use_message_database: Some(true),
            system_language_code: Some("system_language_code_2".to_string()),
            device_model: Some("device_model_2".to_string()),
            ignore_file_names: Some(true),
        };
        let telegram_config = telegram_config.merge(Some(telegram_raw));
        assert_eq!(telegram_config.api_id, "api_id_2");
        assert_eq!(telegram_config.api_hash, "api_hash_2");
        assert_eq!(telegram_config.database_dir, "database_dir_2");
        assert!(telegram_config.use_file_database);
        assert!(telegram_config.use_chat_info_database);
        assert!(telegram_config.use_message_database);
        assert_eq!(
            telegram_config.system_language_code,
            "system_language_code_2"
        );
        assert_eq!(telegram_config.device_model, "device_model_2");
        assert!(!telegram_config.ignore_file_names);
    }

    #[test]
    fn test_get_type() {
        assert_eq!(
            TelegramConfig::get_type(),
            crate::configs::config_type::ConfigType::Telegram
        );
    }
}
