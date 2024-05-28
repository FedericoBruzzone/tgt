fn empty_tgt_folder() {
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    let _ = std::fs::read_dir(format!("{}/.tgt", home));
}

fn move_config_folder_to_home_dottgt() {
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    std::fs::create_dir_all(format!("{}/.tgt/config", home)).unwrap();
    for entry in std::fs::read_dir(format!("{}/config", manifest_dir)).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let new_path = format!("{}/.tgt/config/{}", home, file_name.to_str().unwrap());
        std::fs::copy(path, new_path).unwrap();
    }
}

fn main() {
    empty_tgt_folder();
    move_config_folder_to_home_dottgt();
}
