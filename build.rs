/// Returns (config_dir, tdlib_dir) using legacy ~/.tgt when it exists, else XDG.
fn tgt_build_paths() -> (std::path::PathBuf, std::path::PathBuf) {
    let home = dirs::home_dir().unwrap();
    let legacy = home.join(".tgt");
    if legacy.exists() && legacy.is_dir() {
        (
            legacy.join("config"),
            legacy.join("tdlib"),
        )
    } else {
        let config_base = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
        let data_base = dirs::data_dir().unwrap_or_else(|| home.join(".local").join("share"));
        (
            config_base.join("tgt").join("config"),
            data_base.join("tgt").join("tdlib"),
        )
    }
}

fn ensure_config_folder_exists() {
    let (config_dest, _) = tgt_build_paths();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let config_source = format!("{manifest_dir}/config");

    // Create config directory if it doesn't exist
    std::fs::create_dir_all(&config_dest).unwrap();

    // Copy regular files from config directory ONLY if they don't exist
    for entry in std::fs::read_dir(&config_source).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // Skip directories - we'll handle themes/ separately
        if !path.is_file() {
            continue;
        }

        let file_name = path.file_name().unwrap();
        let dest_file = config_dest.join(file_name);

        // Only copy if file doesn't exist (preserve user customizations)
        if !dest_file.exists() {
            println!("cargo:warning=Creating default config file: {}", dest_file.display());
            std::fs::copy(path, &dest_file).unwrap();
        }
    }

    // Copy themes directory recursively, but don't overwrite existing themes
    let themes_source = format!("{}/themes", config_source);
    let themes_dest = config_dest.join("themes");

    if std::path::Path::new(&themes_source).exists() {
        std::fs::create_dir_all(&themes_dest).unwrap();

        for entry in std::fs::read_dir(&themes_source).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            // Only copy regular files from themes directory
            if !path.is_file() {
                continue;
            }

            let file_name = path.file_name().unwrap();
            let dest_file = themes_dest.join(file_name);

            // Always update theme files (they're defaults, not user customizations)
            std::fs::copy(path, &dest_file).unwrap();
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

    // Only create configs if missing, don't delete existing ones
    ensure_config_folder_exists();

    let (_, tdlib_dest) = tgt_build_paths();
    tdlib_rs::build::build(Some(tdlib_dest.to_string_lossy().into_owned()));

    Ok(())
}
