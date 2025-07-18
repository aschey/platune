use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use eyre::{Context, Result};
use uuid::Uuid;

use super::Config;
use super::config_error::ConfigError;

static CONFIG_FILE: &str = "drive_id";

#[derive(Clone)]
pub struct FileConfig {
    config_path: String,
}

pub fn config_dir() -> Result<PathBuf, ConfigError> {
    let proj_dirs =
        directories::ProjectDirs::from("", "", "platune").ok_or(ConfigError::NoHomeDir)?;
    Ok(proj_dirs.config_dir().to_path_buf())
}

impl FileConfig {
    pub fn try_new() -> Result<Box<dyn Config + Send + Sync>, ConfigError> {
        FileConfig::new_from_path(config_dir()?.join(CONFIG_FILE))
    }

    pub fn new_from_path<P: AsRef<Path>>(
        config_path: P,
    ) -> Result<Box<dyn Config + Send + Sync>, ConfigError> {
        let config_path_ref = config_path.as_ref();
        let config_string = config_path_ref.to_string_lossy().to_string();
        if config_path_ref.to_str().is_none() {
            return Err(ConfigError::InvalidUnicode(config_string));
        }

        if config_path_ref.is_dir() {
            return Err(ConfigError::NotAFile(config_string));
        }

        if !config_path_ref.exists() {
            if let Some(parent) = config_path_ref.parent()
                && let Err(e) = create_dir_all(parent) {
                    return Err(ConfigError::FileCreationFailed(config_string, e));
                }

            if let Err(e) = File::create(config_path_ref) {
                return Err(ConfigError::FileCreationFailed(config_string, e));
            }
        }
        Ok(Box::new(Self {
            config_path: config_string,
        }))
    }
}

impl Config for FileConfig {
    fn get_drive_id(&self) -> Option<Uuid> {
        let mut file = File::open(&self.config_path).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;

        contents.parse::<Uuid>().ok()
    }

    fn set_drive_id(&self, id: Uuid) -> Result<()> {
        let mut file =
            File::create(&self.config_path).wrap_err("Error opening file for writing")?;

        write!(file, "{id:?}").wrap_err(format!(
            "Error writing to config file {:?}",
            self.config_path
        ))
    }
}
