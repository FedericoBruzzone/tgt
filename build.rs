fn empty_tgt_folder() {
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    let _ = std::fs::remove_dir_all(format!("{home}/.tgt/config"));
    let _ = std::fs::remove_dir_all(format!("{home}/.tgt/tdlib"));
}

fn move_config_folder_to_home_dottgt() {
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let config_source = format!("{manifest_dir}/config");
    let config_dest = format!("{home}/.tgt/config");

    std::fs::create_dir_all(&config_dest).unwrap();

    // Copy regular files from config directory
    for entry in std::fs::read_dir(&config_source).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // Skip directories - we'll handle themes/ separately
        if !path.is_file() {
            continue;
        }

        let file_name = path.file_name().unwrap();
        let new_path = format!("{}/{}", config_dest, file_name.to_str().unwrap());
        std::fs::copy(path, new_path).unwrap();
    }

    // Copy themes directory recursively
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
            let new_path = format!("{}/{}", themes_dest, file_name.to_str().unwrap());
            std::fs::copy(path, new_path).unwrap();
        }
    }
}

fn main() -> std::io::Result<()> {
    if cfg!(debug_assertions) {
        tdlib_rs::build::build(None);
        return Ok(());
    }

    empty_tgt_folder();
    move_config_folder_to_home_dottgt();
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    let dest = format!("{home}/.tgt/tdlib");
    tdlib_rs::build::build(Some(dest));

    Ok(())
}
