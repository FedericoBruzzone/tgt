use crate::{
    app_error::AppError,
    configs::{
        self, config_file::ConfigFile, config_type::ConfigType, raw::telegram_raw::TelegramRaw,
    },
};
use std::path::Path;

#[derive(Clone, Debug)]
/// The telegram configuration.
pub struct TelegramConfig {}
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
        Self {}
    }
}

#[cfg(test)]
mod tests {}
