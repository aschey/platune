use std::path::Path;

use sled::IVec;

use crate::database::Database;

static CONFIG_NAMESPACE: &str = "platune-server";
static DRIVE_ID: &str = "drive-id";

pub struct Config {
    sled_db: sled::Db,
    sql_db: Database,
}

impl Config {
    pub fn new(db: &Database) -> Self {
        let proj_dirs = directories::ProjectDirs::from("", "", "platune").unwrap();
        let config_dir = proj_dirs.config_dir();
        let sled_db = sled::open(config_dir).unwrap();
        Self {
            sled_db,
            sql_db: db.clone(),
        }
    }
    pub fn new_from_path<P: AsRef<Path>>(db: &Database, config_dir: P) -> Self {
        let sled_db = sled::open(config_dir).unwrap();

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
                self.sql_db.update_mount(drive_id, path).await;
            }
            None => {
                let id = self.sql_db.add_mount(path).await;
                self.set_drive_id(&id.to_string()[..]);
            }
        };
        let folders = self.sql_db.get_all_folders().await;
        for folder in folders {
            let new_folder = folder.replacen(path, "", 1);
            self.sql_db.update_folder(folder, new_folder).await;
        }
    }

    pub async fn add_folder(&self, path: &str) {
        self.add_folders(vec![path]).await;
    }

    pub async fn add_folders(&self, paths: Vec<&str>) {
        let new_paths = match self.get_registered_mount().await {
            Some(mount) => paths
                .into_iter()
                .map(|path| match path.find(&mount[..]) {
                    Some(0) => path.replacen(&mount[..], "", 1),
                    _ => path.to_owned(),
                })
                .collect(),
            None => paths.into_iter().map(|path| path.to_owned()).collect(),
        };
        self.sql_db.add_folders(new_paths).await;
    }

    pub async fn get_all_folders(&self) -> Vec<String> {
        let folders = self.sql_db.get_all_folders().await;
        match self.get_registered_mount().await {
            Some(mount) => folders
                .into_iter()
                .map(|f| format!("{}{}", mount, f))
                .collect(),
            None => folders,
        }
    }

    pub async fn sync(&self) -> tokio::sync::mpsc::Receiver<f32> {
        let folders = self.get_all_folders().await;
        self.sql_db.sync(folders).await
    }

    pub async fn get_registered_mount(&self) -> Option<String> {
        match self.get_drive_id() {
            Some(drive_id) => {
                let mount = self.sql_db.get_mount(drive_id).await;
                Some(mount)
            }
            None => None,
        }
    }

    fn get_drive_id(&self) -> Option<String> {
        self.get(CONFIG_NAMESPACE, DRIVE_ID)
    }

    fn set_drive_id<V: Into<IVec>>(&self, drive_id: V) {
        self.set(CONFIG_NAMESPACE, DRIVE_ID, drive_id);
    }
}
