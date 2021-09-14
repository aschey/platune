use std::path::{Path, PathBuf};

use crate::database::Database;
use regex::Regex;
use sled::IVec;
use thiserror::Error;

static CONFIG_NAMESPACE: &str = "platune-server";
static DRIVE_ID: &str = "drive-id";

pub struct Config {
    sled_db: sled::Db,
    sql_db: Database,
    delim: &'static str,
    validate_paths: bool,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("{0} is not a valid path")]
    InvalidPath(PathBuf),
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
            delim: if cfg!(windows) { r"\" } else { "/" },
            validate_paths: true,
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

    fn clean_path(&self, path: &str) -> String {
        let mut path = path.replace(self.delim, "/");
        let re = Regex::new(r"/+").unwrap();
        path = re.replace_all(&path, "/").to_string();
        if !path.ends_with("/") {
            path += "/";
        }
        return path;
    }

    pub async fn register_drive<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let path = path.as_ref();
        if self.validate_paths && !path.exists() {
            return Err(ConfigError::InvalidPath(path.to_owned()));
        }

        let path = self.clean_path(&path.to_string_lossy().into_owned());

        match self.get_drive_id() {
            Some(drive_id) => {
                let rows = self.sql_db.update_mount(drive_id, &path).await;
                if rows == 0 {
                    self.set_drive(&path).await;
                }
            }
            None => {
                self.set_drive(&path).await;
            }
        };

        let folders = self.sql_db.get_all_folders().await;
        for folder in folders {
            let new_folder = folder.replacen(&path, "", 1);
            self.sql_db.update_folder(folder, new_folder).await;
        }

        Ok(())
    }

    pub async fn add_folder(&self, path: &str) {
        self.add_folders(vec![path]).await;
    }

    pub async fn add_folders(&self, paths: Vec<&str>) {
        let paths = paths.into_iter().map(|new_path| self.clean_path(new_path));
        let new_paths = match self.get_registered_mount().await {
            Some(mount) => paths
                .map(|path| match path.find(&mount[..]) {
                    Some(0) => path.replacen(&mount[..], "", 1),
                    _ => path.to_owned(),
                })
                .collect::<Vec<_>>(),
            None => paths.map(|path| path.to_owned()).collect(),
        };

        self.sql_db.add_folders(new_paths).await;
    }

    pub async fn get_all_folders(&self) -> Vec<String> {
        let folders = self.sql_db.get_all_folders().await;
        let res = match self.get_registered_mount().await {
            Some(mount) => folders
                .into_iter()
                .map(|f| format!("{}{}", mount, f))
                .collect(),
            None => folders,
        };
        res.into_iter()
            .map(|r| r.replace("/", self.delim))
            .collect()
    }

    pub async fn sync(&self) -> tokio::sync::mpsc::Receiver<f32> {
        let folders = self.get_all_folders().await;
        let mount = self.get_registered_mount().await;
        self.sql_db.sync(folders, mount).await
    }

    pub async fn get_registered_mount(&self) -> Option<String> {
        match self.get_drive_id() {
            Some(drive_id) => self.sql_db.get_mount(drive_id).await,
            None => None,
        }
    }

    fn get_drive_id(&self) -> Option<String> {
        self.get(CONFIG_NAMESPACE, DRIVE_ID)
    }

    async fn set_drive(&self, path: &str) {
        let id = self.sql_db.add_mount(path).await;
        self.set(CONFIG_NAMESPACE, DRIVE_ID, &id.to_string()[..]);
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::Config, database::Database};
    use tempfile::TempDir;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_add_folders() {
        let tempdir = TempDir::new().unwrap();
        let (_, mut config) = setup(&tempdir).await;
        config.delim = r"\";

        config.add_folder(r"test1\").await;
        config.add_folder("test1").await;
        config.add_folder("test2").await;
        config.add_folder(r"test2\\").await;
        let folders = config.get_all_folders().await;

        assert_eq!(vec![r"test1\", r"test2\"], folders);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_change_mount() {
        let tempdir = TempDir::new().unwrap();
        let (_, mut config) = setup(&tempdir).await;
        config.delim = r"\";
        config.validate_paths = false;

        config.register_drive(r"C:\\").await.unwrap();
        config.add_folder(r"C:\test").await;
        config.add_folder(r"C:\\test\\").await;
        let folders1 = config.get_all_folders().await;
        config.register_drive(r"D:\").await.unwrap();
        let folders2 = config.get_all_folders().await;

        assert_eq!(vec![r"C:\test\"], folders1);
        assert_eq!(vec![r"D:\test\"], folders2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_change_mount_after() {
        let tempdir = TempDir::new().unwrap();
        let (_, mut config) = setup(&tempdir).await;
        config.delim = r"\";
        config.validate_paths = false;

        config.add_folder(r"C:\test").await;
        config.add_folder(r"C:\\test\\").await;
        config.register_drive(r"C:\").await.unwrap();
        let folders1 = config.get_all_folders().await;
        config.register_drive(r"D:\").await.unwrap();
        let folders2 = config.get_all_folders().await;

        assert_eq!(vec![r"C:\test\"], folders1);
        assert_eq!(vec![r"D:\test\"], folders2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_multiple_mounts() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut config) = setup(&tempdir).await;
        let config_path2 = tempdir.path().join("platuneconfig2");
        let mut config2 = Config::new_from_path(&db, config_path2);
        config.delim = r"\";
        config.validate_paths = false;
        config2.delim = r"\";
        config2.validate_paths = false;

        config.add_folder(r"C:\test").await;
        config.add_folder(r"C:\\test\\").await;
        config.register_drive(r"C:\").await.unwrap();
        let folders1 = config.get_all_folders().await;
        config2.register_drive(r"D:\").await.unwrap();
        let folders2 = config2.get_all_folders().await;

        assert_eq!(vec![r"C:\test\"], folders1);
        assert_eq!(vec![r"D:\test\"], folders2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_reset_drive_id_if_missing() {
        let tempdir = TempDir::new().unwrap();
        let sql_path = tempdir.path().join("platune.db");
        let config_path = tempdir.path().join("platuneconfig");
        let db = Database::connect(sql_path, true).await;
        db.migrate().await;
        let mut config = Config::new_from_path(&db, config_path.clone());
        config.delim = r"\";
        config.validate_paths = false;

        config.add_folder(r"C:\test").await;
        config.register_drive(r"C:\").await.unwrap();
        drop(config);
        let tempdir2 = TempDir::new().unwrap();
        let sql_path2 = tempdir2.path().join("platune.db");
        let db2 = Database::connect(sql_path2, true).await;
        db2.migrate().await;
        let mut config2 = Config::new_from_path(&db2, config_path);
        config2.delim = r"\";
        config2.validate_paths = false;

        config2.add_folder(r"C:\test").await;
        config2.register_drive(r"C:\").await.unwrap();

        let folders = config2.get_all_folders().await;
        assert_eq!(vec![r"C:\test\"], folders);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_validate_path() {
        let tempdir = TempDir::new().unwrap();
        let (_, mut config) = setup(&tempdir).await;

        config.delim = r"\";

        let res = config.register_drive(r"/some/invalid/path").await;
        assert!(res.is_err());
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
