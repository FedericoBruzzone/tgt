fn ensure_config_folder_exists() {
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let config_source = format!("{manifest_dir}/config");
    let config_dest = format!("{home}/.tgt/config");

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
        let dest_file = format!("{}/{}", config_dest, file_name.to_str().unwrap());

        // Only copy if file doesn't exist (preserve user customizations)
        if !std::path::Path::new(&dest_file).exists() {
            println!("cargo:warning=Creating default config file: {}", dest_file);
            std::fs::copy(path, dest_file).unwrap();
        }
    }

    // Copy themes directory recursively, but don't overwrite existing themes
    let themes_source = format!("{}/themes", config_source);
    let themes_dest = format!("{}/themes", config_dest);

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
            let dest_file = format!("{}/{}", themes_dest, file_name.to_str().unwrap());

            // Always update theme files (they're defaults, not user customizations)
            std::fs::copy(path, dest_file).unwrap();
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

    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    let dest = format!("{home}/.tgt/tdlib");
    tdlib_rs::build::build(Some(dest));

    Ok(())
}
