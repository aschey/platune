use super::Config;
use anyhow::Result;
use std::{cell::RefCell, sync::Mutex};

pub struct MemoryConfig {
    drive_id: Mutex<RefCell<Option<i64>>>,
}

impl MemoryConfig {
    pub fn new_boxed() -> Box<dyn Config + Send + Sync> {
        Box::new(Self {
            drive_id: Mutex::new(RefCell::new(None)),
        })
    }
}

impl Config for MemoryConfig {
    fn get_drive_id(&self) -> Option<i64> {
        *self.drive_id.lock().unwrap().borrow()
    }

    fn set_drive_id(&self, id: i64) -> Result<()> {
        *self.drive_id.lock().unwrap().borrow_mut() = Some(id);
        Ok(())
    }
}
