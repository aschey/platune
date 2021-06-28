use std::path::Path;

use sled::IVec;

use crate::database::Database;

static CONFIG_NAMESPACE: &str = "platune-server";
static DRIVE_ID: &str = "drive-id";

pub struct Config {
    sled_db: sled::Db,
    sql_db: Database,
    replace_delim: bool,
    delim: &'static str,
}

impl Config {
    pub fn new(db: &Database) -> Self {
        let proj_dirs = directories::ProjectDirs::from("", "", "platune").unwrap();
        let config_dir = proj_dirs.config_dir();
        Config::new_from_path(db, config_dir)
    }

    pub fn new_from_path<P: AsRef<Path>>(db: &Database, config_dir: P) -> Self {
        let sled_db = sled::open(config_dir).unwrap();

        Self {
            sled_db,
            sql_db: db.clone(),
            replace_delim: cfg!(windows),
            delim: if cfg!(windows) { "\\" } else { "/" },
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
        let mut path = path.to_owned();
        if !path.ends_with(&self.delim) {
            path += self.delim;
        }
        match self.get_drive_id() {
            Some(drive_id) => {
                self.sql_db.update_mount(drive_id, &path).await;
            }
            None => {
                let id = self.sql_db.add_mount(&path).await;
                self.set_drive_id(&id.to_string()[..]);
            }
        };
        let path = path.replace(self.delim, "/");
        let folders = self.sql_db.get_all_folders().await;
        for folder in folders {
            let new_folder = folder.replacen(&path, "", 1);
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
                .collect::<Vec<_>>(),
            None => paths.into_iter().map(|path| path.to_owned()).collect(),
        };
        let new_paths = new_paths
            .into_iter()
            .map(|mut new_path| {
                if self.replace_delim {
                    new_path = new_path.replace(self.delim, "/");
                }

                if !new_path.ends_with("/") {
                    new_path += "/";
                }
                new_path
            })
            .collect();
        self.sql_db.add_folders(new_paths).await;
    }

    pub async fn get_all_folders(&self) -> Vec<String> {
        let folders = self.sql_db.get_all_folders().await;
        match self.get_registered_mount().await {
            Some(mount) => folders
                .into_iter()
                .map(|mut f| {
                    if self.replace_delim {
                        f = f.replace("/", self.delim);
                    }
                    format!("{}{}", mount, f)
                })
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

    #[cfg(test)]
    fn set_delim(&mut self, delim: &'static str) {
        self.delim = delim;
        self.replace_delim = true;
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::Config, database::Database};
    use tempfile::TempDir;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    pub async fn test_add_folders() {
        let tempdir = TempDir::new().unwrap();
        let (db, config) = setup(&tempdir).await;

        config.add_folder("test1").await;
        config.add_folder("test1").await;
        config.add_folder("test2").await;
        let folders = config.get_all_folders().await;
        db.close().await;
        assert_eq!(vec!["test1/", "test2/"], folders);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    pub async fn test_change_mount() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut config) = setup(&tempdir).await;
        config.set_delim("\\");

        config.register_drive("C:\\").await;
        config.add_folder("C:\\test").await;
        let folders1 = config.get_all_folders().await;
        config.register_drive("D:\\").await;
        let folders2 = config.get_all_folders().await;
        db.close().await;
        assert_eq!(vec!["C:\\test\\"], folders1);
        assert_eq!(vec!["D:\\test\\"], folders2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    pub async fn test_change_mount_after() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut config) = setup(&tempdir).await;
        config.set_delim("\\");

        config.add_folder("C:\\test").await;
        config.register_drive("C:\\").await;
        let folders1 = config.get_all_folders().await;
        config.register_drive("D:\\").await;
        let folders2 = config.get_all_folders().await;
        db.close().await;
        assert_eq!(vec!["C:\\test\\"], folders1);
        assert_eq!(vec!["D:\\test\\"], folders2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    pub async fn test_multiple_mounts() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut config) = setup(&tempdir).await;
        let config_path2 = tempdir.path().join("platuneconfig2");
        let mut config2 = Config::new_from_path(&db, config_path2);
        config.set_delim("\\");
        config2.set_delim("\\");

        config.add_folder("C:\\test").await;
        config.register_drive("C:\\").await;
        let folders1 = config.get_all_folders().await;
        config2.register_drive("D:\\").await;
        let folders2 = config2.get_all_folders().await;
        db.close().await;
        assert_eq!(vec!["C:\\test\\"], folders1);
        assert_eq!(vec!["D:\\test\\"], folders2);
    }

    async fn setup(tempdir: &TempDir) -> (Database, Config) {
        let sql_path = tempdir.path().join("platune.db");
        let config_path = tempdir.path().join("platuneconfig");
        let db = Database::connect(sql_path, true).await;
        db.migrate().await;
        let config = Config::new_from_path(&db, config_path);
        (db, config)
    }
}
