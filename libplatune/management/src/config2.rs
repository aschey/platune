// use anyhow::{Context, Result};
// use std::{
//     cell::RefCell,
//     fs::{self, File},
//     io::{Error, Read, Write},
//     path::Path,
//     sync::{Arc, Mutex},
// };
// use thiserror::Error;

// static CONFIG_FILE: &str = "drive_id";

// #[derive(Clone)]
// pub struct Config {
//     config_path: String,
// }

// pub struct MemoryConfig {
//     drive_id: Mutex<RefCell<Option<i64>>>,
// }

// #[derive(Error, Debug)]
// pub enum ConfigError {
//     #[error("Unable to locate a valid home directory")]
//     NoHomeDir,
//     #[error("Failed to create file {0}: {1}")]
//     FileCreationFailed(String, Error),
//     #[error("{0} is not a file")]
//     NotAFile(String),
//     #[error("{0} contains invalid unicode")]
//     InvalidUnicode(String),
// }

// pub trait ConfigTrait {
//     fn get_drive_id(&self) -> Option<i64>;
//     fn set_drive_id(&self, id: i64) -> Result<()>;
// }

// impl Config {
//     pub fn try_new() -> Result<Box<dyn ConfigTrait + Send + Sync>, ConfigError> {
//         let proj_dirs =
//             directories::ProjectDirs::from("", "", "platune").ok_or(ConfigError::NoHomeDir)?;
//         let config_dir = proj_dirs.config_dir();
//         Config::new_from_path(config_dir.join(CONFIG_FILE))
//     }

//     pub fn new_from_path<P: AsRef<Path>>(
//         config_path: P,
//     ) -> Result<Box<dyn ConfigTrait + Send + Sync>, ConfigError> {
//         let config_path_ref = config_path.as_ref();
//         let config_string = config_path_ref.to_string_lossy().to_string();
//         if config_path_ref.to_str().is_none() {
//             return Err(ConfigError::InvalidUnicode(config_string));
//         }

//         if config_path_ref.is_dir() {
//             return Err(ConfigError::NotAFile(config_string));
//         }

//         if !config_path_ref.exists() {
//             if let Some(parent) = config_path_ref.parent() {
//                 if let Err(e) = fs::create_dir_all(parent) {
//                     return Err(ConfigError::FileCreationFailed(config_string, e));
//                 }
//             }

//             if let Err(e) = File::create(&config_path_ref) {
//                 return Err(ConfigError::FileCreationFailed(config_string, e));
//             }
//         }
//         Ok(Box::new(Self {
//             config_path: config_string,
//         }))
//     }
// }

// impl ConfigTrait for Config {
//     fn get_drive_id(&self) -> Option<i64> {
//         let mut file = File::open(&self.config_path).ok()?;
//         let mut contents = String::new();
//         file.read_to_string(&mut contents).ok()?;

//         contents.parse::<i64>().ok()
//     }

//     fn set_drive_id(&self, id: i64) -> Result<()> {
//         let mut file =
//             File::create(&self.config_path).with_context(|| "Error opening file for writing")?;

//         write!(file, "{id:?}")
//             .with_context(|| format!("Error writing to config file {:?}", self.config_path))
//     }
// }

// impl MemoryConfig {
//     pub fn new() -> Box<dyn ConfigTrait + Send + Sync> {
//         Box::new(Self {
//             drive_id: Mutex::new(RefCell::new(None)),
//         })
//     }
// }

// impl ConfigTrait for MemoryConfig {
//     fn get_drive_id(&self) -> Option<i64> {
//         *self.drive_id.lock().unwrap().borrow()
//     }

//     fn set_drive_id(&self, id: i64) -> Result<()> {
//         *self.drive_id.lock().unwrap().borrow_mut() = Some(id);
//         Ok(())
//     }
// }