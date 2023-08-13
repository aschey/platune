mod config_error;
mod file_config;
mod memory_config;

use eyre::Result;
pub use file_config::*;
pub use memory_config::*;
use uuid::Uuid;

pub trait Config {
    fn get_drive_id(&self) -> Option<Uuid>;
    fn set_drive_id(&self, id: Uuid) -> Result<()>;
}
