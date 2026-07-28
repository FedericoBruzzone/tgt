use crate::{
    app_error::AppError,
    configs::{
        self,
        config_file::ConfigFile,
        config_type::ConfigType,
        raw::telegram_raw::{ProxyRaw, TelegramRaw},
    },
    utils,
};
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
/// The proxy type.
pub enum ProxyType {
    Socks5,
    Http,
    Mtproto,
}

#[derive(Clone, Debug)]
/// The proxy configuration.
pub struct ProxyConfig {
    /// Proxy type.
    pub r#type: ProxyType,
    /// Proxy server domain or IP address.
    pub server: String,
    /// Proxy server port.
    pub port: i32,
    /// Username for SOCKS5 or HTTP proxy (empty if unused).
    pub username: String,
    /// Password for SOCKS5 or HTTP proxy (empty if unused).
    pub password: String,
    /// HTTP proxy only: true if the proxy supports only HTTP requests.
    pub http_only: bool,
    /// MTProto proxy only: hex-encoded secret (empty if unused).
    pub secret: String,
}

impl From<ProxyRaw> for ProxyConfig {
    fn from(raw: ProxyRaw) -> Self {
        let proxy_type = match raw.r#type.as_deref() {
            Some("http") => ProxyType::Http,
            Some("mtproto") => ProxyType::Mtproto,
            _ => ProxyType::Socks5,
        };
        Self {
            r#type: proxy_type,
            server: raw.server.unwrap_or_default(),
            port: raw.port.unwrap_or(1080),
            username: raw.username.unwrap_or_default(),
            password: raw.password.unwrap_or_default(),
            http_only: raw.http_only.unwrap_or(false),
            secret: raw.secret.unwrap_or_default(),
        }
    }
}

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
    /// The verbosity level of the logging.
    pub verbosity_level: i32,
    /// The path to the working directory.
    pub log_path: String,
    /// A flag that indicates if the log to stderr should be also redirected.
    pub redirect_stderr: bool,
    /// Optional proxy configuration.
    pub proxy: Option<ProxyConfig>,
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
                    let abs = if Path::new(&database_dir).is_absolute() {
                        PathBuf::from(&database_dir)
                    } else if let Some(legacy) = utils::tgt_legacy_dir() {
                        legacy.join(&database_dir)
                    } else {
                        utils::tgt_data_dir().unwrap().join(&database_dir)
                    };
                    if !abs.exists() {
                        std::fs::create_dir_all(&abs).unwrap();
                    }
                    self.database_dir = abs.to_string_lossy().to_string();
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
                if let Some(verbosity_level) = _other.verbosity_level {
                    self.verbosity_level = verbosity_level;
                }
                if let Some(log_path) = _other.log_path {
                    let abs = if Path::new(&log_path).is_absolute() {
                        PathBuf::from(&log_path)
                    } else if let Some(legacy) = utils::tgt_legacy_dir() {
                        legacy.join(&log_path)
                    } else {
                        utils::tgt_state_dir().unwrap().join(&log_path)
                    };
                    if let Some(parent) = abs.parent() {
                        if !parent.exists() {
                            std::fs::create_dir_all(parent).unwrap();
                        }
                    }
                    self.log_path = abs.to_string_lossy().to_string();
                }
                if let Some(redirect_stderr) = _other.redirect_stderr {
                    self.redirect_stderr = redirect_stderr;
                }
                if let Some(proxy_raw) = _other.proxy {
                    let config: ProxyConfig = proxy_raw.into();
                    if !config.server.is_empty() {
                        tracing::info!(
                            "Proxy configured: {:?} {}:{}",
                            config.r#type,
                            config.server,
                            config.port
                        );
                        self.proxy = Some(config);
                    } else {
                        tracing::info!("Proxy section present but server is empty, skipping");
                        self.proxy = None;
                    }
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
        let (database_dir, log_path) = if cfg!(debug_assertions) {
            let base = utils::tgt_dir().unwrap();
            (
                base.join(raw.database_dir.as_deref().unwrap_or(".data/tg"))
                    .to_string_lossy()
                    .to_string(),
                base.join(
                    raw.log_path
                        .as_deref()
                        .unwrap_or(".data/tdlib_rs/tdlib_rs.log"),
                )
                .to_string_lossy()
                .to_string(),
            )
        } else if let Some(legacy) = utils::tgt_legacy_dir() {
            (
                legacy
                    .join(raw.database_dir.as_deref().unwrap_or(".data/tg"))
                    .to_string_lossy()
                    .to_string(),
                legacy
                    .join(
                        raw.log_path
                            .as_deref()
                            .unwrap_or(".data/tdlib_rs/tdlib_rs.log"),
                    )
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            (
                utils::tgt_data_dir()
                    .unwrap()
                    .join("tg")
                    .to_string_lossy()
                    .to_string(),
                utils::tgt_state_dir()
                    .unwrap()
                    .join("tdlib_rs")
                    .join("tdlib_rs.log")
                    .to_string_lossy()
                    .to_string(),
            )
        };

        if !Path::new(&database_dir).exists() {
            std::fs::create_dir_all(&database_dir).unwrap();
        }
        if let Some(parent) = PathBuf::from(&log_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
        }

        let proxy = raw.proxy.and_then(|p| {
            let config: ProxyConfig = p.into();
            if config.server.is_empty() {
                None
            } else {
                Some(config)
            }
        });

        Self {
            api_id: raw.api_id.unwrap(),
            api_hash: raw.api_hash.unwrap(),
            database_dir,
            use_file_database: raw.use_file_database.unwrap(),
            use_chat_info_database: raw.use_chat_info_database.unwrap(),
            use_message_database: raw.use_message_database.unwrap(),
            system_language_code: raw.system_language_code.unwrap(),
            device_model: raw.device_model.unwrap(),
            verbosity_level: raw.verbosity_level.unwrap(),
            log_path,
            redirect_stderr: raw.redirect_stderr.unwrap(),
            proxy,
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
            database_dir: Some(".data/tg".to_string()),
            use_file_database: Some(true),
            use_chat_info_database: Some(true),
            use_message_database: Some(true),
            system_language_code: Some("system_language_code".to_string()),
            device_model: Some("device_model".to_string()),
            verbosity_level: Some(1),
            log_path: Some(".data/tdlib_rs/tdlib_rs.log".to_string()),
            redirect_stderr: Some(true),
            proxy: None,
        };
        let telegram_config = TelegramConfig::from(telegram_raw);
        assert_eq!(telegram_config.api_id, "api_id");
        assert_eq!(telegram_config.api_hash, "api_hash");
        assert_eq!(
            telegram_config.database_dir,
            utils::tgt_dir()
                .unwrap()
                .join(".data/tg")
                .to_string_lossy()
                .to_string()
        );
        assert!(telegram_config.use_file_database);
        assert!(telegram_config.use_chat_info_database);
        assert!(telegram_config.use_message_database);
        assert_eq!(telegram_config.system_language_code, "system_language_code");
        assert_eq!(telegram_config.device_model, "device_model");
        assert_eq!(telegram_config.verbosity_level, 1);
        assert_eq!(
            telegram_config.log_path,
            utils::tgt_dir()
                .unwrap()
                .join(".data/tdlib_rs/tdlib_rs.log")
                .to_string_lossy()
                .to_string()
        );
        assert!(telegram_config.redirect_stderr);
    }

    #[test]
    fn test_telegram_merge() {
        let mut telegram_config = TelegramConfig {
            api_id: "api_id".to_string(),
            api_hash: "api_hash".to_string(),
            database_dir: ".data/tg".to_string(),
            use_file_database: false,
            use_chat_info_database: false,
            use_message_database: false,
            system_language_code: "system_language_code".to_string(),
            device_model: "device_model".to_string(),
            verbosity_level: 1,
            log_path: ".data/tdlib_rs/tdlib_rs.log".to_string(),
            redirect_stderr: false,
            proxy: None,
        };
        let telegram_raw = TelegramRaw {
            api_id: Some("api_id_2".to_string()),
            api_hash: Some("api_hash_2".to_string()),
            database_dir: None,
            use_file_database: Some(true),
            use_chat_info_database: Some(true),
            use_message_database: Some(true),
            system_language_code: Some("system_language_code_2".to_string()),
            device_model: Some("device_model_2".to_string()),
            verbosity_level: Some(2),
            log_path: None,
            redirect_stderr: Some(true),
            proxy: None,
        };
        let telegram_config = telegram_config.merge(Some(telegram_raw));
        assert_eq!(telegram_config.api_id, "api_id_2");
        assert_eq!(telegram_config.api_hash, "api_hash_2");
        assert_eq!(telegram_config.database_dir, ".data/tg");
        assert!(telegram_config.use_file_database);
        assert!(telegram_config.use_chat_info_database);
        assert!(telegram_config.use_message_database);
        assert_eq!(
            telegram_config.system_language_code,
            "system_language_code_2"
        );
        assert_eq!(telegram_config.device_model, "device_model_2");
        assert_eq!(telegram_config.verbosity_level, 2);
        assert_eq!(telegram_config.log_path, ".data/tdlib_rs/tdlib_rs.log");
        assert!(telegram_config.redirect_stderr);
    }

    #[test]
    fn test_telegram_merge_none() {
        let mut telegram_config = TelegramConfig {
            api_id: "api_id".to_string(),
            api_hash: "api_hash".to_string(),
            database_dir: ".data/tg".to_string(),
            use_file_database: false,
            use_chat_info_database: false,
            use_message_database: false,
            system_language_code: "system_language_code".to_string(),
            device_model: "device_model".to_string(),
            verbosity_level: 1,
            log_path: ".data/tdlib_rs/tdlib_rs.log".to_string(),
            redirect_stderr: false,
            proxy: None,
        };
        let telegram_config = telegram_config.merge(None);
        assert_eq!(telegram_config.api_id, "api_id");
        assert_eq!(telegram_config.api_hash, "api_hash");
        assert_eq!(telegram_config.database_dir, ".data/tg");
        assert!(!telegram_config.use_file_database);
        assert!(!telegram_config.use_chat_info_database);
        assert!(!telegram_config.use_message_database);
        assert_eq!(telegram_config.system_language_code, "system_language_code");
        assert_eq!(telegram_config.device_model, "device_model");
        assert_eq!(telegram_config.verbosity_level, 1);
        assert_eq!(telegram_config.log_path, ".data/tdlib_rs/tdlib_rs.log");
        assert!(!telegram_config.redirect_stderr);
    }

    #[test]
    fn test_telegram_merge_partial() {
        let mut telegram_config = TelegramConfig {
            api_id: "api_id".to_string(),
            api_hash: "api_hash".to_string(),
            database_dir: ".data/tg".to_string(),
            use_file_database: false,
            use_chat_info_database: false,
            use_message_database: false,
            system_language_code: "system_language_code".to_string(),
            device_model: "device_model".to_string(),
            verbosity_level: 1,
            log_path: ".data/tdlib_rs/tdlib_rs.log".to_string(),
            redirect_stderr: false,
            proxy: None,
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
            verbosity_level: None,
            log_path: None,
            redirect_stderr: Some(true),
            proxy: None,
        };
        let telegram_config = telegram_config.merge(Some(telegram_raw));
        assert_eq!(telegram_config.api_id, "api_id_2");
        assert_eq!(telegram_config.api_hash, "api_hash");
        assert_eq!(telegram_config.database_dir, ".data/tg");
        assert!(!telegram_config.use_file_database);
        assert!(!telegram_config.use_chat_info_database);
        assert!(!telegram_config.use_message_database);
        assert_eq!(telegram_config.system_language_code, "system_language_code");
        assert_eq!(telegram_config.device_model, "device_model");
        assert_eq!(telegram_config.verbosity_level, 1);
        assert_eq!(telegram_config.log_path, ".data/tdlib_rs/tdlib_rs.log");
        assert!(telegram_config.redirect_stderr);
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
            database_dir: ".data/tg".to_string(),
            use_file_database: false,
            use_chat_info_database: false,
            use_message_database: false,
            system_language_code: "system_language_code".to_string(),
            device_model: "device_model".to_string(),
            verbosity_level: 1,
            log_path: ".data/tdlib_rs/tdlib_rs.log".to_string(),
            redirect_stderr: false,
            proxy: None,
        };
        let telegram_raw = TelegramRaw {
            api_id: Some("api_id_2".to_string()),
            api_hash: Some("api_hash_2".to_string()),
            database_dir: None,
            use_file_database: Some(true),
            use_chat_info_database: Some(true),
            use_message_database: Some(true),
            system_language_code: Some("system_language_code_2".to_string()),
            device_model: Some("device_model_2".to_string()),
            verbosity_level: Some(2),
            log_path: None,
            redirect_stderr: Some(true),
            proxy: None,
        };
        let telegram_config = telegram_config.merge(Some(telegram_raw));
        assert_eq!(telegram_config.api_id, "api_id_2");
        assert_eq!(telegram_config.api_hash, "api_hash_2");
        assert_eq!(telegram_config.database_dir, ".data/tg");
        assert!(telegram_config.use_file_database);
        assert!(telegram_config.use_chat_info_database);
        assert!(telegram_config.use_message_database);
        assert_eq!(
            telegram_config.system_language_code,
            "system_language_code_2"
        );
        assert_eq!(telegram_config.device_model, "device_model_2");
        assert_eq!(telegram_config.verbosity_level, 2);
        assert_eq!(telegram_config.log_path, ".data/tdlib_rs/tdlib_rs.log");
        assert!(telegram_config.redirect_stderr);
    }

    #[test]
    fn test_get_type() {
        assert_eq!(
            TelegramConfig::get_type(),
            crate::configs::config_type::ConfigType::Telegram
        );
    }

    // ── Proxy tests ──

    use crate::configs::{
        custom::telegram_custom::{ProxyConfig, ProxyType},
        raw::telegram_raw::ProxyRaw,
    };

    #[test]
    fn test_proxy_from_raw_socks5() {
        let raw = ProxyRaw {
            r#type: Some("socks5".into()),
            server: Some("10.0.0.1".into()),
            port: Some(1080),
            username: Some("user".into()),
            password: Some("pass".into()),
            http_only: None,
            secret: None,
        };
        let cfg: ProxyConfig = raw.into();
        assert_eq!(cfg.r#type, ProxyType::Socks5);
        assert_eq!(cfg.server, "10.0.0.1");
        assert_eq!(cfg.port, 1080);
        assert_eq!(cfg.username, "user");
        assert_eq!(cfg.password, "pass");
    }

    #[test]
    fn test_proxy_from_raw_http() {
        let raw = ProxyRaw {
            r#type: Some("http".into()),
            server: Some("proxy.example.com".into()),
            port: Some(3128),
            username: Some("u".into()),
            password: Some("p".into()),
            http_only: Some(true),
            secret: None,
        };
        let cfg: ProxyConfig = raw.into();
        assert_eq!(cfg.r#type, ProxyType::Http);
        assert_eq!(cfg.server, "proxy.example.com");
        assert_eq!(cfg.port, 3128);
        assert!(cfg.http_only);
    }

    #[test]
    fn test_proxy_from_raw_mtproto() {
        let raw = ProxyRaw {
            r#type: Some("mtproto".into()),
            server: Some("mtp.example.com".into()),
            port: Some(443),
            username: None,
            password: None,
            http_only: None,
            secret: Some("deadbeef".into()),
        };
        let cfg: ProxyConfig = raw.into();
        assert_eq!(cfg.r#type, ProxyType::Mtproto);
        assert_eq!(cfg.secret, "deadbeef");
    }

    #[test]
    fn test_proxy_from_raw_unknown_type_defaults_to_socks5() {
        let raw = ProxyRaw {
            r#type: Some("invalid".into()),
            server: Some("127.0.0.1".into()),
            port: Some(9999),
            username: None,
            password: None,
            http_only: None,
            secret: None,
        };
        let cfg: ProxyConfig = raw.into();
        assert_eq!(cfg.r#type, ProxyType::Socks5);
    }

    #[test]
    fn test_proxy_from_raw_defaults() {
        let raw = ProxyRaw {
            r#type: None,
            server: None,
            port: None,
            username: None,
            password: None,
            http_only: None,
            secret: None,
        };
        let cfg: ProxyConfig = raw.into();
        assert_eq!(cfg.r#type, ProxyType::Socks5);
        assert_eq!(cfg.server, "");
        assert_eq!(cfg.port, 1080);
        assert_eq!(cfg.username, "");
        assert_eq!(cfg.password, "");
        assert!(!cfg.http_only);
        assert_eq!(cfg.secret, "");
    }

    #[test]
    fn test_telegram_from_raw_with_proxy() {
        let telegram_raw = TelegramRaw {
            api_id: Some("1".into()),
            api_hash: Some("h".into()),
            database_dir: Some(".data/tg".into()),
            use_file_database: Some(true),
            use_chat_info_database: Some(true),
            use_message_database: Some(true),
            system_language_code: Some("en".into()),
            device_model: Some("Desktop".into()),
            verbosity_level: Some(2),
            log_path: Some(".data/tdlib_rs/tdlib_rs.log".into()),
            redirect_stderr: Some(false),
            proxy: Some(ProxyRaw {
                r#type: Some("socks5".into()),
                server: Some("10.0.0.1".into()),
                port: Some(1080),
                username: None,
                password: None,
                http_only: None,
                secret: None,
            }),
        };
        let cfg = TelegramConfig::from(telegram_raw);
        assert!(cfg.proxy.is_some());
        let p = cfg.proxy.unwrap();
        assert_eq!(p.r#type, ProxyType::Socks5);
        assert_eq!(p.server, "10.0.0.1");
        assert_eq!(p.port, 1080);
    }

    #[test]
    fn test_telegram_from_raw_proxy_empty_server() {
        let telegram_raw = TelegramRaw {
            api_id: Some("1".into()),
            api_hash: Some("h".into()),
            database_dir: Some(".data/tg".into()),
            use_file_database: Some(true),
            use_chat_info_database: Some(true),
            use_message_database: Some(true),
            system_language_code: Some("en".into()),
            device_model: Some("Desktop".into()),
            verbosity_level: Some(2),
            log_path: Some(".data/tdlib_rs/tdlib_rs.log".into()),
            redirect_stderr: Some(false),
            proxy: Some(ProxyRaw {
                r#type: Some("socks5".into()),
                server: Some("".into()),
                port: Some(1080),
                username: None,
                password: None,
                http_only: None,
                secret: None,
            }),
        };
        let cfg = TelegramConfig::from(telegram_raw);
        assert!(cfg.proxy.is_none());
    }

    #[test]
    fn test_telegram_merge_sets_proxy() {
        let mut cfg = TelegramConfig {
            api_id: "1".into(),
            api_hash: "h".into(),
            database_dir: ".data/tg".into(),
            use_file_database: true,
            use_chat_info_database: true,
            use_message_database: true,
            system_language_code: "en".into(),
            device_model: "Desktop".into(),
            verbosity_level: 2,
            log_path: ".data/tdlib_rs/tdlib_rs.log".into(),
            redirect_stderr: false,
            proxy: None,
        };
        let raw = TelegramRaw {
            api_id: None,
            api_hash: None,
            database_dir: None,
            use_file_database: None,
            use_chat_info_database: None,
            use_message_database: None,
            system_language_code: None,
            device_model: None,
            verbosity_level: None,
            log_path: None,
            redirect_stderr: None,
            proxy: Some(ProxyRaw {
                r#type: Some("http".into()),
                server: Some("proxy:8080".into()),
                port: Some(8080),
                username: None,
                password: None,
                http_only: Some(true),
                secret: None,
            }),
        };
        let merged = cfg.merge(Some(raw));
        assert!(merged.proxy.is_some());
        let p = merged.proxy.unwrap();
        assert_eq!(p.r#type, ProxyType::Http);
        assert_eq!(p.server, "proxy:8080");
        assert!(p.http_only);
    }

    #[test]
    fn test_telegram_merge_proxy_empty_server_clears() {
        let mut cfg = TelegramConfig {
            api_id: "1".into(),
            api_hash: "h".into(),
            database_dir: ".data/tg".into(),
            use_file_database: true,
            use_chat_info_database: true,
            use_message_database: true,
            system_language_code: "en".into(),
            device_model: "Desktop".into(),
            verbosity_level: 2,
            log_path: ".data/tdlib_rs/tdlib_rs.log".into(),
            redirect_stderr: false,
            proxy: Some(ProxyConfig {
                r#type: ProxyType::Socks5,
                server: "10.0.0.1".into(),
                port: 1080,
                username: "".into(),
                password: "".into(),
                http_only: false,
                secret: "".into(),
            }),
        };
        let raw = TelegramRaw {
            api_id: None,
            api_hash: None,
            database_dir: None,
            use_file_database: None,
            use_chat_info_database: None,
            use_message_database: None,
            system_language_code: None,
            device_model: None,
            verbosity_level: None,
            log_path: None,
            redirect_stderr: None,
            proxy: Some(ProxyRaw {
                r#type: Some("socks5".into()),
                server: Some("".into()),
                port: Some(1080),
                username: None,
                password: None,
                http_only: None,
                secret: None,
            }),
        };
        let merged = cfg.merge(Some(raw));
        assert!(merged.proxy.is_none());
    }
}
