mod config_error;
mod file_config;
mod memory_config;

pub use file_config::*;
pub use memory_config::*;

use anyhow::Result;

pub trait Config {
    fn get_drive_id(&self) -> Option<i64>;
    fn set_drive_id(&self, id: i64) -> Result<()>;
}
