fn empty_tgt_folder() {
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    let _ = std::fs::remove_dir_all(format!("{home}/.tgt/config"));
    let _ = std::fs::remove_dir_all(format!("{home}/.tgt/tdlib"));
}

fn move_config_folder_to_home_dottgt() {
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    std::fs::create_dir_all(format!("{home}/.tgt/config")).unwrap();
    for entry in std::fs::read_dir(format!("{manifest_dir}/config")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let new_path = format!("{home}/.tgt/config/{}", file_name.to_str().unwrap());
        std::fs::copy(path, new_path).unwrap();
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
