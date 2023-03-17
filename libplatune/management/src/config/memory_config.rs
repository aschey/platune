use super::Config;
use eyre::Result;
use std::{cell::RefCell, sync::Mutex};
use uuid::Uuid;

pub struct MemoryConfig {
    drive_id: Mutex<RefCell<Option<Uuid>>>,
}

impl MemoryConfig {
    pub fn new_boxed() -> Box<dyn Config + Send + Sync> {
        Box::new(Self {
            drive_id: Mutex::new(RefCell::new(None)),
        })
    }
}

impl Config for MemoryConfig {
    fn get_drive_id(&self) -> Option<Uuid> {
        *self.drive_id.lock().unwrap().borrow()
    }

    fn set_drive_id(&self, id: Uuid) -> Result<()> {
        *self.drive_id.lock().unwrap().borrow_mut() = Some(id);
        Ok(())
    }
}
