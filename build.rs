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
