fn empty_tgt_folder() {
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    std::fs::remove_dir_all(format!("{}/.tgt/config", home)).unwrap();
    std::fs::remove_dir_all(format!("{}/.tgt/tdlib", home)).unwrap();
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

// #[cfg(not(any(feature = "docs", feature = "pkg-config", feature = "download-tdlib")))]
// /// Copy all files from a directory to another.
// fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
//     std::fs::create_dir_all(&dst)?;
//     for entry in std::fs::read_dir(src)? {
//         let entry = entry?;
//         let ty = entry.file_type()?;
//         if ty.is_dir() {
//             copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
//         } else {
//             std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
//         }
//     }
//     Ok(())
// }
//
// #[cfg(not(any(feature = "docs", feature = "pkg-config", feature = "download-tdlib")))]
// /// Copy all the tdlib folder find in the LOCAL_TDLIB_PATH environment variable to the OUT_DIR/tdlib folder
// fn copy_local_tdlib() {
//     match env::var("LOCAL_TDLIB_PATH") {
//         Ok(tdlib_path) => {
//             let out_dir = env::var("OUT_DIR").unwrap();
//             let prefix = format!("{}/tdlib", out_dir);
//             copy_dir_all(Path::new(&tdlib_path), Path::new(&prefix)).unwrap();
//         }
//         Err(_) => {
//             panic!("The LOCAL_TDLIB_PATH env variable must be set to the path of the tdlib folder");
//         }
//     };
// }
//
// #[cfg(not(any(feature = "docs", feature = "pkg-config")))]
// /// Build the project using the generic build configuration.
// /// The current supported platforms are:
// /// - Linux x86_64
// /// - Windows x86_64
// /// - MacOS x86_64
// /// - MacOS aarch64
// fn generic_build() {
//     let out_dir = env::var("OUT_DIR").unwrap();
//     println!("cargo:warning=out_dir: {}", out_dir);
//     let prefix = format!("{}/tdlib", out_dir);
//     let include_dir = format!("{}/include", prefix);
//     let lib_dir = format!("{}/lib", prefix);
//     let lib_path = {
//         #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
//         {
//             format!("{}/libtdjson.so.{}", lib_dir, TDLIB_VERSION)
//         }
//         #[cfg(any(
//             all(target_os = "macos", target_arch = "x86_64"),
//             all(target_os = "macos", target_arch = "aarch64")
//         ))]
//         {
//             format!("{}/libtdjson.{}.dylib", lib_dir, TDLIB_VERSION)
//         }
//         #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
//         {
//             format!(r"{}\tdjson.lib", lib_dir)
//         }
//     };
//
//     if !std::path::PathBuf::from(lib_path.clone()).exists() {
//         panic!("tdjson shared library not found at {}", lib_path);
//     }
//
//     #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
//     {
//         let bin_dir = format!(r"{}\bin", prefix);
//         println!("cargo:rustc-link-search=native={}", bin_dir);
//     }
//
//     println!("cargo:rustc-link-search=native={}", lib_dir);
//     println!("cargo:include={}", include_dir);
//     println!("cargo:rustc-link-lib=dylib=tdjson");
//     println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir);
// }
//
// #[cfg(feature = "download-tdlib")]
// fn download_tdlib() {
//     let base_url = "https://github.com/FedericoBruzzone/tdlib-rs/releases/download";
//     let url = format!(
//         "{}/v{}/tdlib-{}-{}-{}.zip",
//         base_url,
//         // env!("CARGO_PKG_VERSION"),
//         "1.0.3",
//         TDLIB_VERSION,
//         std::env::var("CARGO_CFG_TARGET_OS").unwrap(),
//         std::env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
//     );
//     // let target_os = if cfg!(target_os = "windows") {
//     //     "Windows"
//     // } else if cfg!(target_os = "linux") {
//     //     "Linux"
//     // } else if cfg!(target_os = "macos") {
//     //     "macOS"
//     // } else {
//     //     ""
//     // };
//     // let target_arch = if cfg!(target_arch = "x86_64") {
//     //     "X64"
//     // } else if cfg!(target_arch = "aarch64") {
//     //     "ARM64"
//     // } else {
//     //     ""
//     // };
//     // let url = format!(
//     //     "{}/test/{}-{}-TDLib-{}.zip",
//     //     base_url, target_os, target_arch, "2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09"
//     // );
//
//     let out_dir = env::var("OUT_DIR").unwrap();
//     let tdlib_dir = format!("{}/tdlib", &out_dir);
//     let zip_path = format!("{}.zip", &tdlib_dir);
//
//     // Create the request
//     let response = reqwest::blocking::get(&url).unwrap();
//
//     // Check if the response status is successful
//     if response.status().is_success() {
//         // Create a file to write to
//         let mut dest = File::create(&zip_path).unwrap();
//
//         // Get the response bytes and write to the file
//         let content = response.bytes().unwrap();
//         std::io::copy(&mut content.as_ref(), &mut dest).unwrap();
//     } else {
//         panic!(
//             "[{}] Failed to download file: {}\n{}\n{}",
//             "Your OS or architecture may be unsupported.",
//             "Please try using the `pkg-config` or `local-tdlib` features.",
//             response.status(),
//             &url
//         )
//     }
//
//     let mut archive = zip::ZipArchive::new(File::open(&zip_path).unwrap()).unwrap();
//
//     for i in 0..archive.len() {
//         let mut file = archive.by_index(i).unwrap();
//         let outpath = Path::new(&out_dir).join(file.name());
//
//         if (*file.name()).ends_with('/') {
//             std::fs::create_dir_all(&outpath).unwrap();
//         } else {
//             if let Some(p) = outpath.parent() {
//                 if !p.exists() {
//                     std::fs::create_dir_all(p).unwrap();
//                 }
//             }
//             let mut outfile = File::create(&outpath).unwrap();
//             std::io::copy(&mut file, &mut outfile).unwrap();
//         }
//
//         // Get and set permissions
//         #[cfg(unix)]
//         {
//             use std::os::unix::fs::PermissionsExt;
//             if let Some(mode) = file.unix_mode() {
//                 std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).unwrap();
//             }
//         }
//     }
//
//     let _ = std::fs::remove_file(&zip_path);
// }

fn main() -> std::io::Result<()> {
    if cfg!(debug_assertions) {
        tdlib_rs::build::check_features();
        tdlib_rs::build::set_rerun_if();

        #[cfg(feature = "pkg-config")]
        tdlib_rs::build::build_pkg_config();

        #[cfg(feature = "download-tdlib")]
        tdlib_rs::build::build_download_tdlib(None);

        #[cfg(not(any(feature = "docs", feature = "pkg-config", feature = "download-tdlib")))]
        tdlib_rs::build::build_local_tdlib();

        return Ok(());
    }

    empty_tgt_folder();
    move_config_folder_to_home_dottgt();

    #[cfg(feature = "pkg-config")]
    tdlib_rs::build::build_pkg_config();

    #[cfg(feature = "download-tdlib")]
    {
        let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
        let dest = format!("{}/.tgt/tdlib", home);
        tdlib_rs::build::build_download_tdlib(Some(dest));
    }

    #[cfg(not(any(feature = "docs", feature = "pkg-config", feature = "download-tdlib")))]
    tdlib_rs::build::build_local_tdlib();

    // // Prevent linking libraries to avoid documentation failure
    // #[cfg(not(feature = "docs"))]
    // {
    //     // It requires the following variables to be set:
    //     // - export PKG_CONFIG_PATH=$HOME/lib/tdlib/lib/pkgconfig/:$PKG_CONFIG_PATH
    //     // - export LD_LIBRARY_PATH=$HOME/lib/tdlib/lib/:$LD_LIBRARY_PATH
    //     #[cfg(feature = "pkg-config")]
    //     system_deps::Config::new().probe().unwrap();
    //
    //     #[cfg(feature = "download-tdlib")]
    //     download_tdlib();
    //
    //     #[cfg(not(feature = "pkg-config"))]
    //     {
    //         #[cfg(not(feature = "download-tdlib"))]
    //         copy_local_tdlib();
    //         generic_build();
    //     }
    // }

    Ok(())
}
