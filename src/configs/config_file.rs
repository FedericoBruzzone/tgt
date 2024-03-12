use {
        crate::configs::{self, config_type::ConfigType, TGT_CONFIG_HOME, TGT_PROGRAM_NAME},
        lazy_static::lazy_static,
        serde::de::DeserializeOwned,
        std::path::PathBuf,
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
pub trait ConfigFile: Sized + Default + Clone {
        /// The raw configuration type.
        /// This type is used to parse the configuration file and must implement the `DeserializeOwned` trait.
        type Raw: Into<Self> + DeserializeOwned;
        /// Get the configuration type.
        ///
        /// # Returns
        /// The configuration type.
        fn get_type() -> ConfigType;
        /// Get the configuration of the specified type.
        ///
        /// # Returns
        /// The configuration of the specified type.
        fn get_config() -> Self {
                if Self::override_fields() {
                        let mut default = Self::default();
                        default.merge(Self::deserialize_custom_config::<Self::Raw>(
                                Self::get_type().as_default_filename().as_str(),
                        ))
                } else {
                        Self::deserialize_config_or_default::<Self::Raw, Self>(
                                Self::get_type().as_default_filename().as_str(),
                        )
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
        fn search_config_directories(file_name: &str) -> Option<PathBuf> {
                CONFIG_DIR_HIERARCHY
                        .iter()
                        .map(|path| path.join(file_name))
                        .find(|path| path.exists())
        }
        /// Deserialize a custom configuration file into a configuration struct.
        /// This function searches the configuration directories for the specified file name and attempts to parse it.
        /// If the file is found and parsed successfully, the parsed configuration is returned.
        /// If the file is not found or cannot be parsed, `None` is returned.
        ///
        /// # Arguments
        /// * `file_name` - The name of the file (including the file extension) to search for in the configuration directories.
        ///
        /// # Returns
        /// The parsed configuration or `None` if the file is not found or cannot be parsed.
        fn deserialize_custom_config<R>(file_name: &str) -> Option<R>
        where
                R: DeserializeOwned,
        {
                match Self::search_config_directories(file_name) {
                        Some(file_path) => match configs::deserialize_to_config::<R>(&file_path) {
                                Ok(s) => {
                                        tracing::info!("Loaded config from {}", file_path.display());
                                        Some(s)
                                }
                                Err(e) => {
                                        tracing::error!("Failed to parse {}: {}", file_name, e);
                                        eprintln!("Failed to parse {}: {}", file_name, e);
                                        None
                                }
                        },
                        None => {
                                tracing::warn!("No config file found for {}", file_name);
                                None
                        }
                }
        }
        /// Deserialize a configuration file into a configuration struct or return the default configuration.
        /// This function searches the configuration directories for the specified file name and attempts to parse it.
        /// If the file is found and parsed successfully, the parsed configuration is returned.
        /// If the file is not found or cannot be parsed, the default configuration is returned.
        ///
        /// # Arguments
        /// * `file_name` - The name of the file (including the file extension) to search for in the configuration directories.
        ///
        /// # Returns
        /// The parsed configuration or the default configuration if the file is not found or cannot be parsed.
        fn deserialize_config_or_default<R, S>(file_name: &str) -> S
        where
                R: DeserializeOwned + Into<S>,
                S: std::default::Default,
        {
                // [TODO] Handle CLI arguments
                match Self::search_config_directories(file_name) {
                        Some(file_path) => match configs::deserialize_to_config_into::<R, S>(&file_path) {
                                Ok(s) => {
                                        tracing::info!("Loaded config from {}", file_path.display());
                                        s
                                }
                                Err(e) => {
                                        tracing::error!("Failed to parse {}: {}", file_name, e);
                                        eprintln!("Failed to parse {}: {}", file_name, e);
                                        S::default()
                                }
                        },
                        None => {
                                tracing::warn!("No config file found for {}", file_name);
                                S::default()
                        }
                }
        }
        #[allow(unused_variables)]
        /// Merge the configuration with another configuration.
        /// If the other configuration is `None`, the current configuration is returned.
        /// This function is used to merge the default configuration with a custom configuration.
        /// The custom configuration is used to override the default configuration.
        ///
        /// # Arguments
        /// * `other` - The other configuration to merge with the current configuration.
        ///
        /// # Returns
        /// The merged configuration.
        fn merge(&mut self, other: Option<Self::Raw>) -> Self {
                self.clone()
        }
        /// Allow the fields of the default configuration to be overridden by the fields of the custom configuration when merging the configurations.
        ///
        /// # Returns
        /// `true` if the fields of the default configuration can be overridden by the fields of the custom configuration, `false` otherwise.
        fn override_fields() -> bool {
                false
        }
}
