//! CLI subcommand handlers: clear, init-config.
//! Uses the same XDG/legacy path logic as the rest of the app.
//! Clear only removes config-related dirs; never deletes paths that contain the
//! project root (Cargo.toml, .git) or tdlib-rs.

use std::io::{self, Write};
use std::path::Path;

use crate::utils::{tgt_config_dir_path, tgt_data_dir, tgt_state_dir};

/// Refuse to remove a directory that looks like a repo root or contains tdlib-rs,
/// so that `tgt clear` never deletes the project or build artifacts.
fn is_unsafe_to_remove(path: &Path) -> bool {
    if !path.exists() || !path.is_dir() {
        return false;
    }
    path.join("Cargo.toml").exists() || path.join(".git").exists() || path.join("tdlib-rs").exists()
}

/// Clear config, data, and/or logs. Confirms unless `yes` is true.
pub fn run_clear(
    config: bool,
    data: bool,
    logs: bool,
    all: bool,
    yes: bool,
) -> std::io::Result<()> {
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
        if is_unsafe_to_remove(path) {
            eprintln!(
                "Refusing to remove {} at {}: directory contains Cargo.toml, .git, or tdlib-rs. \
                 Only config-related content should be cleared.",
                label,
                path.display()
            );
            std::process::exit(1);
        }
        remove_dir_all(path).map_err(|e| {
            io::Error::other(format!(
                "Failed to remove {} at {}: {}",
                label,
                path.display(),
                e
            ))
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

/// Copy default config into user config dir from the embedded bundle (always available, e.g. after
/// `cargo install`). If `overwrite_existing` is false, only write when the destination is missing.
fn copy_bundled_config_to_user(overwrite_existing: bool, verbose: bool) -> std::io::Result<()> {
    let config_dest = crate::utils::tgt_config_dir()?;
    for (rel_path, content) in crate::bundled_config::bundled_config_files() {
        let dest = config_dest.join(rel_path);
        if overwrite_existing || !dest.exists() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, content)?;
            if verbose {
                println!("Created/updated: {}", dest.display());
            }
        }
    }
    if verbose {
        println!("Config initialized at: {}", config_dest.display());
    }
    Ok(())
}

/// Ensure default config files exist in the user config dir (e.g. XDG). If any are missing,
/// copy them from the bundled config so the app can start. Silent; no output. Call at startup
/// before loading configs so that after deleting ~/.config/tgt we still get a working config.
pub fn ensure_default_config_files_if_missing() -> std::io::Result<()> {
    copy_bundled_config_to_user(false, false)
}

/// Copy default config to user config dir (from embedded bundle). If `force` is false, only copy
/// missing files. With `force`, overwrite existing files (no backup in this implementation).
pub fn run_init_config(force: bool) -> std::io::Result<()> {
    copy_bundled_config_to_user(force, true)
}
