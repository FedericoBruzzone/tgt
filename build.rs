/// Returns (config_dir, tdlib_dir). Config may be legacy or XDG; tdlib is always the standard
/// data dir (e.g. ~/Library/Application Support/tgt/tdlib on macOS) so the folder is always
/// created and the app finds it after cleanup (when legacy is gone).
fn tgt_build_paths() -> (std::path::PathBuf, std::path::PathBuf) {
    let home = dirs::home_dir().unwrap();
    let config_base = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
    let data_base = dirs::data_dir().unwrap_or_else(|| home.join(".local").join("share"));
    let standard_config = config_base.join("tgt").join("config");
    let standard_tdlib = data_base.join("tgt").join("tdlib");
    let legacy = home.join(".tgt");
    let config_dir = if legacy.exists() && legacy.is_dir() {
        legacy.join("config")
    } else {
        standard_config
    };
    // Always use standard tdlib path so it exists after clear; runtime uses tgt_data_dir() which matches.
    (config_dir, standard_tdlib)
}

/// Copy default config files from manifest config/ into dest. Only copy files that don't exist (preserve user customizations).
fn copy_default_config_into(config_dest: &std::path::Path, manifest_dir: &str) {
    let config_source = format!("{manifest_dir}/config");

    std::fs::create_dir_all(config_dest).unwrap();

    for entry in std::fs::read_dir(&config_source).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = path.file_name().unwrap();
        let dest_file = config_dest.join(file_name);
        if !dest_file.exists() {
            println!(
                "cargo:warning=Creating default config file: {}",
                dest_file.display()
            );
            std::fs::copy(path, &dest_file).unwrap();
        }
    }

    let themes_source = format!("{}/themes", config_source);
    let themes_dest = config_dest.join("themes");
    if std::path::Path::new(&themes_source).exists() {
        std::fs::create_dir_all(&themes_dest).unwrap();
        for entry in std::fs::read_dir(&themes_source).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_owned();
                std::fs::copy(&path, themes_dest.join(&file_name)).unwrap();
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    // chafa-dyn and chafa-static are not supported on Windows ARM; fail the build if enabled there
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let feature_chafa_dyn = std::env::var("CARGO_CFG_FEATURE_CHAFA_DYN").unwrap_or_default();
    let feature_chafa_static = std::env::var("CARGO_CFG_FEATURE_CHAFA_STATIC").unwrap_or_default();
    if os == "windows" && arch == "aarch64" {
        if feature_chafa_dyn == "1" {
            panic!("the chafa-dyn feature is not supported on Windows ARM; build without --features chafa-dyn");
        }
        if feature_chafa_static == "1" {
            panic!("the chafa-static feature is not supported on Windows ARM; build without --features chafa-static");
        }
    }

    if cfg!(debug_assertions) {
        tdlib_rs::build::build(None);
        return Ok(());
    }

    // Copy default configs to both legacy (if present) and XDG so that after deleting ~/.tgt the app still finds configs.
    // Never copy into the repo (CARGO_MANIFEST_DIR) so we don't overwrite tracked config files.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = std::path::Path::new(&manifest_dir);
    let (config_dest_legacy_or_xdg, _) = tgt_build_paths();
    if !config_dest_legacy_or_xdg.starts_with(manifest_path) {
        copy_default_config_into(&config_dest_legacy_or_xdg, &manifest_dir);
    }
    let home = dirs::home_dir().unwrap();
    let config_base = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
    let xdg_config = config_base.join("tgt").join("config");
    if xdg_config != config_dest_legacy_or_xdg && !xdg_config.starts_with(manifest_path) {
        copy_default_config_into(&xdg_config, &manifest_dir);
    }

    let (_, tdlib_dest) = tgt_build_paths();
    std::fs::create_dir_all(&tdlib_dest).unwrap();
    tdlib_rs::build::build(Some(tdlib_dest.to_string_lossy().into_owned()));

    Ok(())
}
