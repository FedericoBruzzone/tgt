use crate::{
    app_error::AppError,
    configs::{
        self, config_file::ConfigFile, config_type::ConfigType, raw::telegram_raw::TelegramRaw,
    },
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
    fn from(_raw: TelegramRaw) -> Self {
        Self {
            api_id: _raw.api_id.unwrap(),
            api_hash: _raw.api_hash.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::{
        config_file::ConfigFile, custom::telegram_custom::TelegramConfig,
        raw::telegram_raw::TelegramRaw,
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
        };
        let telegram_config = TelegramConfig::from(telegram_raw);
        assert_eq!(telegram_config.api_id, "api_id");
        assert_eq!(telegram_config.api_hash, "api_hash");
    }

    #[test]
    fn test_telegram_merge() {
        let mut telegram_config = TelegramConfig {
            api_id: "api_id".to_string(),
            api_hash: "api_hash".to_string(),
        };
        let telegram_raw = TelegramRaw {
            api_id: Some("api_id_2".to_string()),
            api_hash: Some("api_hash_2".to_string()),
        };
        let telegram_config = telegram_config.merge(Some(telegram_raw));
        assert_eq!(telegram_config.api_id, "api_id_2");
        assert_eq!(telegram_config.api_hash, "api_hash_2");
    }

    #[test]
    fn test_telegram_merge_none() {
        let mut telegram_config = TelegramConfig {
            api_id: "api_id".to_string(),
            api_hash: "api_hash".to_string(),
        };
        let telegram_config = telegram_config.merge(None);
        assert_eq!(telegram_config.api_id, "api_id");
        assert_eq!(telegram_config.api_hash, "api_hash");
    }

    #[test]
    fn test_telegram_merge_partial() {
        let mut telegram_config = TelegramConfig {
            api_id: "api_id".to_string(),
            api_hash: "api_hash".to_string(),
        };
        let telegram_raw = TelegramRaw {
            api_id: Some("api_id_2".to_string()),
            api_hash: None,
        };
        let telegram_config = telegram_config.merge(Some(telegram_raw));
        assert_eq!(telegram_config.api_id, "api_id_2");
        assert_eq!(telegram_config.api_hash, "api_hash");
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
        };
        let telegram_raw = TelegramRaw {
            api_id: Some("api_id_2".to_string()),
            api_hash: Some("api_hash_2".to_string()),
        };
        let telegram_config = telegram_config.merge(Some(telegram_raw));
        assert_eq!(telegram_config.api_id, "api_id_2");
        assert_eq!(telegram_config.api_hash, "api_hash_2");
    }

    #[test]
    fn test_get_type() {
        assert_eq!(
            TelegramConfig::get_type(),
            crate::configs::config_type::ConfigType::Telegram
        );
    }
}
