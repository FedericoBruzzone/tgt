use std::{env, io, path::PathBuf};

pub mod custom;
pub mod raw;

pub mod config_dir_hierarchy;
pub mod config_type;

pub const TGT_PROGRAM_NAME: &str = "tgt";
pub const TGT_CONFIG_HOME: &str = "TGT_CONFIG_HOME";

pub fn project_dir() -> io::Result<PathBuf> {
  env::current_dir()
}

pub fn default_config_dir() -> io::Result<PathBuf> {
  Ok(project_dir()?.join("config"))
}
