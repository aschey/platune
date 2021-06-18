use sled::IVec;

use crate::database::Database;

pub struct Config {
    sled_db: sled::Db,
    sql_db: Database,
}

impl Config {
    pub fn new(db: &Database) -> Self {
        let base_dirs = directories::BaseDirs::new().unwrap();
        let sled_file = base_dirs.config_dir().join("platune/sled");

        let sled_db = sled::open(sled_file).unwrap();
        Self {
            sled_db,
            sql_db: db.clone(),
        }
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

    pub async fn register_drive(&self, path: &str) {
        match self.get_drive_id() {
            Some(drive_id) => {
                self.sql_db.update_mount(&drive_id[..], path).await;
            }
            None => {
                let id = self.sql_db.add_mount(path).await;
                self.set("platune-server", "os-id", &id.to_string()[..]);
            }
        }
    }

    pub fn get_drive_id(&self) -> Option<String> {
        self.get("platune-server", "os-id")
    }
}
