use dirs;
use std::path::{Path, PathBuf};
use std::{env, io};

pub const TGT: &str = "tgt";
pub const TGT_CONFIG_DIR: &str = "TGT_CONFIG_DIR";
/// In debug builds, config/data/state live under this dir so that `tgt clear` never
/// removes the project root or tdlib-rs; only config-related content is cleared.
const TGT_DEBUG_DEV_DIR: &str = ".tgt-dev";

/// Resolve config, data, and state directories from base paths and legacy presence.
/// Used for testing and by the public path getters.
/// Returns (config_dir, data_dir, state_dir).
#[doc(hidden)]
pub fn resolve_tgt_paths(
    home: &Path,
    config_base: &Path,
    data_base: &Path,
    state_base: &Path,
    legacy_exists: bool,
) -> (PathBuf, PathBuf, PathBuf) {
    if legacy_exists {
        let legacy = home.join(format!(".{}", TGT));
        (legacy.join("config"), legacy.clone(), legacy.join(".data"))
    } else {
        (
            config_base.join(TGT).join("config"),
            data_base.join(TGT),
            state_base.join(TGT),
        )
    }
}

/// Single source of truth: legacy dir path if it exists and is a directory.
/// Does not consider debug_assertions (callers that need test isolation use tgt_legacy_dir()).
fn legacy_dir_if_exists() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let legacy = home.join(format!(".{}", TGT));
    if legacy.exists() && legacy.is_dir() {
        Some(legacy)
    } else {
        None
    }
}

/// Returns the legacy tgt directory (`$HOME/.tgt`) if it exists, otherwise None.
/// In debug mode (e.g. tests) returns None so that XDG/current-dir paths are used.
pub fn tgt_legacy_dir() -> Option<PathBuf> {
    if cfg!(debug_assertions) {
        return None;
    }
    legacy_dir_if_exists()
}

/// Returns the config directory path without creating it. Use for read-only hierarchy.
pub fn tgt_config_dir_path() -> io::Result<PathBuf> {
    if cfg!(debug_assertions) {
        return Ok(env::current_dir()?.join(TGT_DEBUG_DEV_DIR).join("config"));
    }
    let dirs = tgt_config_dir_paths_ordered()?;
    dirs.into_iter()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No config directory path"))
}

/// Returns user config directory paths in search order: legacy first if it exists, then XDG.
/// Used to build CONFIG_DIR_HIERARCHY so legacy ~/.tgt/config is always tried before XDG.
pub fn tgt_config_dir_paths_ordered() -> io::Result<Vec<PathBuf>> {
    if cfg!(debug_assertions) {
        return Ok(vec![env::current_dir()?
            .join(TGT_DEBUG_DEV_DIR)
            .join("config")]);
    }
    let home = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME directory not found"))?;
    let config_base = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
    let data_base = dirs::data_dir().unwrap_or_else(|| home.join(".local").join("share"));
    let state_base = dirs::state_dir().unwrap_or_else(|| home.join(".local").join("state"));
    let mut paths = Vec::new();
    if let Some(legacy) = legacy_dir_if_exists() {
        paths.push(legacy.join("config"));
    }
    let (xdg_config, _, _) = resolve_tgt_paths(&home, &config_base, &data_base, &state_base, false);
    paths.push(xdg_config);
    Ok(paths)
}

/// Get the configuration directory (XDG_CONFIG_HOME/tgt/config or ~/.tgt/config if legacy).
/// Creates the directory only when used for writing; does not create ~/.tgt in non-legacy case.
pub fn tgt_config_dir() -> io::Result<PathBuf> {
    let config_dir = tgt_config_dir_path()?;
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
        tracing::info!("Created config directory at: {}", config_dir.display());
    }
    Ok(config_dir)
}

/// Get the data directory (XDG_DATA_HOME/tgt or ~/.tgt if legacy).
pub fn tgt_data_dir() -> io::Result<PathBuf> {
    if cfg!(debug_assertions) {
        return Ok(env::current_dir()?.join(TGT_DEBUG_DEV_DIR).join("data"));
    }
    let home = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME directory not found"))?;
    let config_base = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
    let data_base = dirs::data_dir().unwrap_or_else(|| home.join(".local").join("share"));
    let state_base = dirs::state_dir().unwrap_or_else(|| home.join(".local").join("state"));
    let legacy_exists = legacy_dir_if_exists().is_some();
    let (_, data_dir, _) =
        resolve_tgt_paths(&home, &config_base, &data_base, &state_base, legacy_exists);
    Ok(data_dir)
}

/// Get the state directory (XDG_STATE_HOME/tgt or ~/.tgt/.data if legacy).
pub fn tgt_state_dir() -> io::Result<PathBuf> {
    if cfg!(debug_assertions) {
        return Ok(env::current_dir()?.join(TGT_DEBUG_DEV_DIR).join("state"));
    }
    let home = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME directory not found"))?;
    let config_base = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
    let data_base = dirs::data_dir().unwrap_or_else(|| home.join(".local").join("share"));
    let state_base = dirs::state_dir().unwrap_or_else(|| home.join(".local").join("state"));
    let legacy_exists = legacy_dir_if_exists().is_some();
    let (_, _, state_dir) =
        resolve_tgt_paths(&home, &config_base, &data_base, &state_base, legacy_exists);
    Ok(state_dir)
}

/// Get the project directory (legacy ~/.tgt or XDG data base). Creates it only in legacy case.
/// Prefer tgt_config_dir(), tgt_data_dir(), tgt_state_dir() for specific uses.
pub fn tgt_dir() -> io::Result<PathBuf> {
    if cfg!(debug_assertions) {
        return env::current_dir();
    }
    // Legacy: use ~/.tgt only if it already exists; do not create ~/.tgt when using XDG.
    if let Some(legacy) = tgt_legacy_dir() {
        return Ok(legacy);
    }
    tgt_data_dir()
}

/// Fail with an error message and exit the application.
///
/// # Arguments
/// * `msg` - A string slice that holds the error message.
/// * `e` - A generic type that holds the error.
///
/// # Returns
/// * `!` - This function does not return a value.
fn fail_with<E: std::fmt::Debug>(msg: &str, e: E) -> ! {
    eprintln!("[ERROR]: {msg} {e:?}");
    std::process::exit(1);
}

/// Unwrap a result or fail with an error message.
/// This function will unwrap a result and return the value if it is Ok.
/// If the result is an error, this function will fail with an error message.
///
/// # Arguments
/// * `result` - A result that holds a value or an error.
/// * `msg` - A string slice that holds the error message.
///
/// # Returns
/// * `T` - The value if the result is Ok.
pub fn unwrap_or_fail<T, E: std::fmt::Debug>(result: Result<T, E>, msg: &str) -> T {
    match result {
        Ok(v) => v,
        Err(e) => fail_with(msg, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn resolve_tgt_paths_uses_legacy_when_exists() {
        let home = Path::new("/home/user");
        let config_base = Path::new("/home/user/.config");
        let data_base = Path::new("/home/user/.local/share");
        let state_base = Path::new("/home/user/.local/state");
        let (config, data, state) =
            resolve_tgt_paths(home, config_base, data_base, state_base, true);
        assert_eq!(config, Path::new("/home/user/.tgt/config"));
        assert_eq!(data, Path::new("/home/user/.tgt"));
        assert_eq!(state, Path::new("/home/user/.tgt/.data"));
    }

    #[test]
    fn resolve_tgt_paths_uses_xdg_when_no_legacy() {
        let home = Path::new("/home/user");
        let config_base = Path::new("/home/user/.config");
        let data_base = Path::new("/home/user/.local/share");
        let state_base = Path::new("/home/user/.local/state");
        let (config, data, state) =
            resolve_tgt_paths(home, config_base, data_base, state_base, false);
        assert_eq!(config, Path::new("/home/user/.config/tgt/config"));
        assert_eq!(data, Path::new("/home/user/.local/share/tgt"));
        assert_eq!(state, Path::new("/home/user/.local/state/tgt"));
    }

    // Tests that touch .tgt or config dirs MUST set HOME to a temp dir (and restore after).
    // Never modify or remove the real ~/.tgt so that running tests does not affect the user's config.

    #[test]
    fn tgt_legacy_dir_none_when_no_dot_tgt() {
        // When HOME points to a dir without .tgt, legacy dir should be None.
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().to_path_buf();
        assert!(!home.join(".tgt").exists());
        let old_home = env::var("HOME").ok();
        env::set_var("HOME", &home);
        let result = tgt_legacy_dir();
        let used_home = dirs::home_dir();
        if let Some(ref h) = old_home {
            env::set_var("HOME", h);
        } else {
            env::remove_var("HOME");
        }
        // Only assert if dirs used our temp as HOME
        if used_home.as_ref() == Some(&home) {
            assert!(result.is_none());
        }
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)] // tgt_legacy_dir() returns None in debug so tests use current_dir
    fn tgt_legacy_dir_some_when_dot_tgt_exists() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        let dot_tgt = home.join(".tgt");
        assert!(
            dot_tgt.starts_with(temp.path()),
            "must only create .tgt under temp dir, never real HOME"
        );
        std::fs::create_dir_all(&dot_tgt).unwrap();
        env::set_var("HOME", home);
        let result = tgt_legacy_dir();
        env::remove_var("HOME");
        #[cfg(target_os = "linux")]
        assert_eq!(result.as_deref(), Some(dot_tgt.as_path()));
        let _ = result;
    }

    /// When legacy ~/.tgt exists, config dir paths must list legacy first so config discovery finds it.
    /// Only runs in release (in debug we use current_dir and legacy is ignored).
    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn tgt_config_dir_paths_ordered_prefers_legacy_when_exists() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        let dot_tgt = home.join(".tgt");
        let legacy_config = dot_tgt.join("config");
        assert!(
            legacy_config.starts_with(temp.path()),
            "must only create under temp dir, never real HOME"
        );
        std::fs::create_dir_all(&legacy_config).unwrap();
        env::set_var("HOME", home);
        let ordered = tgt_config_dir_paths_ordered().unwrap();
        env::remove_var("HOME");
        assert!(!ordered.is_empty(), "should return at least one path");
        // When dirs::home_dir() respects our HOME, first path must be legacy so config is discovered
        if ordered[0].starts_with(home) {
            assert_eq!(
                ordered[0], legacy_config,
                "when ~/.tgt exists, first config dir must be legacy so existing config is discovered"
            );
        }
        // Regardless: first path must look like a legacy config dir (ends with .tgt/config)
        assert!(
            ordered[0].to_string_lossy().ends_with(".tgt/config"),
            "first path should be legacy config dir, got {}",
            ordered[0].display()
        );
    }
}
