use sled::IVec;
use std::path::Path;
use thiserror::Error;

static CONFIG_NAMESPACE: &str = "platune-server";
static DRIVE_ID: &str = "drive-id";

#[derive(Clone)]
pub struct Config {
    sled_db: sled::Db,
}
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Unable to locate a valid home directory")]
    NoHomeDir,
}

impl Config {
    pub fn try_new() -> Result<Self, ConfigError> {
        let proj_dirs =
            directories::ProjectDirs::from("", "", "platune").ok_or(ConfigError::NoHomeDir)?;
        let config_dir = proj_dirs.config_dir();
        Ok(Config::new_from_path(config_dir))
    }

    pub fn new_from_path<P: AsRef<Path>>(config_dir: P) -> Self {
        let sled_db = sled::open(config_dir).unwrap();

        Self { sled_db }
    }

    pub fn set<K: AsRef<[u8]>, N: AsRef<[u8]>, V: Into<IVec>>(
        &self,
        namespace: N,
        key: K,
        value: V,
    ) {
        self.sled_db
            .open_tree(namespace)
            .unwrap()
            .insert(key, value)
            .unwrap();
    }

    pub fn get<K: AsRef<[u8]>, N: AsRef<[u8]>>(&self, namespace: N, key: K) -> Option<String> {
        let val = self.sled_db.open_tree(namespace).unwrap().get(key).unwrap();
        match val {
            None => None,
            Some(val) => Some(std::str::from_utf8(&val).unwrap().to_owned()),
        }
    }

    pub(crate) fn get_drive_id(&self) -> Option<String> {
        self.get(CONFIG_NAMESPACE, DRIVE_ID)
    }

    pub(crate) fn set_drive_id(&self, id: &str) {
        self.set(CONFIG_NAMESPACE, DRIVE_ID, id);
    }
}
