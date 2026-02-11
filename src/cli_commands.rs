//! CLI subcommand handlers: clear, init-config.
//! Uses the same XDG/legacy path logic as the rest of the app.

use std::io::{self, Write};
use std::path::Path;

use crate::utils::{tgt_config_dir_path, tgt_data_dir, tgt_state_dir};

/// Clear config, data, and/or logs. Confirms unless `yes` is true.
pub fn run_clear(config: bool, data: bool, logs: bool, all: bool, yes: bool) -> std::io::Result<()> {
    let do_config = config || all;
    let do_data = data || all;
    let do_logs = logs || all;
    if !do_config && !do_data && !do_logs {
        eprintln!("Specify at least one of --config, --data, --logs, or --all.");
        std::process::exit(1);
    }

    let mut to_remove: Vec<(String, std::path::PathBuf)> = Vec::new();
    if do_config {
        if let Ok(p) = tgt_config_dir_path() {
            if p.exists() {
                to_remove.push(("Config".into(), p));
            }
        }
    }
    if do_data {
        if let Ok(p) = tgt_data_dir() {
            if p.exists() {
                to_remove.push(("Data".into(), p));
            }
        }
    }
    if do_logs {
        if let Ok(p) = tgt_state_dir() {
            if p.exists() {
                to_remove.push(("Logs/state".into(), p));
            }
        }
    }

    if to_remove.is_empty() {
        println!("Nothing to clear (target directories do not exist).");
        return Ok(());
    }

    if !yes {
        println!("The following will be removed:");
        for (label, p) in &to_remove {
            println!("  {}: {}", label, p.display());
        }
        print!("Proceed? [y/N] ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") && !line.trim().eq_ignore_ascii_case("yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    for (label, path) in &to_remove {
        remove_dir_all(path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to remove {} at {}: {}", label, path.display(), e),
            )
        })?;
        println!("Removed {}: {}", label, path.display());
    }
    Ok(())
}

fn remove_dir_all(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        Ok(())
    }
}

/// Copy bundled default config to user config dir. If `force` is false, only copy missing files.
/// With `force`, overwrite existing files (no backup in this implementation).
pub fn run_init_config(force: bool) -> std::io::Result<()> {
    // CARGO_MANIFEST_DIR is set at compile time by Cargo, so the path is baked into the binary.
    let manifest_dir = option_env!("CARGO_MANIFEST_DIR").unwrap_or(".");
    let config_source = Path::new(manifest_dir).join("config");
    if !config_source.exists() {
        eprintln!("Default config source not found: {}", config_source.display());
        std::process::exit(1);
    }

    let config_dest = crate::utils::tgt_config_dir()?;
    std::fs::create_dir_all(&config_dest)?;

    // Copy top-level config files
    for entry in std::fs::read_dir(&config_source)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().unwrap();
        let dest = config_dest.join(name);
        if force || !dest.exists() {
            std::fs::copy(&path, &dest)?;
            println!("Created/updated: {}", dest.display());
        }
    }

    // Copy themes
    let themes_src = config_source.join("themes");
    let themes_dest = config_dest.join("themes");
    if themes_src.exists() {
        std::fs::create_dir_all(&themes_dest)?;
        for entry in std::fs::read_dir(&themes_src)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap();
                let dest = themes_dest.join(name);
                if force || !dest.exists() {
                    std::fs::copy(&path, &dest)?;
                    println!("Created/updated theme: {}", dest.display());
                }
            }
        }
    }

    println!("Config initialized at: {}", config_dest.display());
    Ok(())
}
