use regex::Regex;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::sync::mpsc;

use crate::{
    config::Config,
    database::{Database, EntryType, LookupEntry, SearchOptions, SearchRes},
};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("{0} is not a valid path")]
    InvalidPath(PathBuf),
}

#[derive(Clone)]
pub struct Manager {
    db: Database,
    config: Config,
    validate_paths: bool,
    delim: &'static str,
}

impl Manager {
    pub fn new(db: &Database, config: &Config) -> Self {
        Self {
            db: db.clone(),
            config: config.clone(),
            validate_paths: true,
            delim: if cfg!(windows) { r"\" } else { "/" },
        }
    }

    pub async fn register_drive<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let path = path.as_ref();
        if self.validate_paths && !path.exists() {
            return Err(ConfigError::InvalidPath(path.to_owned()));
        }

        let path = self.clean_path(&path.to_string_lossy().into_owned());

        match self.config.get_drive_id() {
            Some(drive_id) => {
                let rows = self.db.update_mount(drive_id, &path).await;
                if rows == 0 {
                    self.set_drive(&path).await;
                }
            }
            None => {
                self.set_drive(&path).await;
            }
        };

        let folders = self.db.get_all_folders().await;
        for folder in folders {
            let new_folder = folder.replacen(&path, "", 1);
            self.db.update_folder(folder, new_folder).await;
        }

        Ok(())
    }

    pub async fn add_folder(&self, path: &str) {
        self.add_folders(vec![path]).await;
    }

    pub async fn add_folders(&self, paths: Vec<&str>) {
        let new_paths = self.replace_prefix(paths).await;
        self.db.add_folders(new_paths).await;
    }

    pub async fn get_all_folders(&self) -> Vec<String> {
        let folders = self.db.get_all_folders().await;
        self.expand_paths(folders).await
    }

    pub async fn sync(&self) -> mpsc::Receiver<f32> {
        let folders = self.get_all_folders().await;
        let mount = self.get_registered_mount().await;
        self.db.sync(folders, mount).await
    }

    async fn replace_prefix(&self, paths: Vec<&str>) -> Vec<String> {
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
        return new_paths;
    }

    pub async fn expand_paths(&self, folders: Vec<String>) -> Vec<String> {
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

    pub async fn get_registered_mount(&self) -> Option<String> {
        match self.config.get_drive_id() {
            Some(drive_id) => self.db.get_mount(drive_id).await,
            None => None,
        }
    }

    pub async fn lookup(
        &self,
        correlation_ids: Vec<i32>,
        entry_type: EntryType,
    ) -> Vec<LookupEntry> {
        let mut res = self.db.lookup(correlation_ids, entry_type).await;
        if let Some(mount) = self.get_registered_mount().await {
            let mount = Path::new(&mount);
            for entry in &mut res {
                entry.path = mount
                    .join(entry.path.to_owned())
                    .to_str()
                    .unwrap()
                    .to_owned()
            }
        }
        res
    }

    pub async fn search(&self, query: &str, options: SearchOptions<'_>) -> Vec<SearchRes> {
        self.db.search(query, options).await
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

    async fn set_drive(&self, path: &str) {
        let id = self.db.add_mount(path).await;
        self.config.set_drive_id(&id.to_string()[..]);
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::Config, database::Database};
    use tempfile::TempDir;

    use super::Manager;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_add_folders() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut config) = setup(&tempdir).await;
        config.delim = r"\";

        config.add_folder(r"test1\").await;
        config.add_folder("test1").await;
        config.add_folder("test2").await;
        config.add_folder(r"test2\\").await;
        let folders = config.get_all_folders().await;

        db.close().await;
        tempdir.close().unwrap();

        assert_eq!(vec![r"test1\", r"test2\"], folders);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_change_mount() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut manager) = setup(&tempdir).await;
        manager.delim = r"\";
        manager.validate_paths = false;

        manager.register_drive(r"C:\\").await.unwrap();
        manager.add_folder(r"C:\test").await;
        manager.add_folder(r"C:\\test\\").await;
        let folders1 = manager.get_all_folders().await;
        manager.register_drive(r"D:\").await.unwrap();
        let folders2 = manager.get_all_folders().await;

        db.close().await;
        tempdir.close().unwrap();

        assert_eq!(vec![r"C:\test\"], folders1);
        assert_eq!(vec![r"D:\test\"], folders2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_change_mount_after() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut manager) = setup(&tempdir).await;
        manager.delim = r"\";
        manager.validate_paths = false;

        manager.add_folder(r"C:\test").await;
        manager.add_folder(r"C:\\test\\").await;
        manager.register_drive(r"C:\").await.unwrap();
        let folders1 = manager.get_all_folders().await;
        manager.register_drive(r"D:\").await.unwrap();
        let folders2 = manager.get_all_folders().await;

        db.close().await;
        tempdir.close().unwrap();

        assert_eq!(vec![r"C:\test\"], folders1);
        assert_eq!(vec![r"D:\test\"], folders2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_multiple_mounts() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut manager) = setup(&tempdir).await;
        let config_path2 = tempdir.path().join("platuneconfig2");
        let config2 = Config::new_from_path(config_path2);
        let mut manager2 = Manager::new(&db, &config2);
        manager.delim = r"\";
        manager.validate_paths = false;
        manager2.delim = r"\";
        manager2.validate_paths = false;

        manager.add_folder(r"C:\test").await;
        manager.add_folder(r"C:\\test\\").await;
        manager.register_drive(r"C:\").await.unwrap();
        let folders1 = manager.get_all_folders().await;
        manager2.register_drive(r"D:\").await.unwrap();
        let folders2 = manager2.get_all_folders().await;

        db.close().await;
        tempdir.close().unwrap();

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
        let config = Config::new_from_path(config_path.clone());
        let mut manager = Manager::new(&db, &config);
        manager.delim = r"\";
        manager.validate_paths = false;

        manager.add_folder(r"C:\test").await;
        manager.register_drive(r"C:\").await.unwrap();

        let tempdir2 = TempDir::new().unwrap();
        let sql_path2 = tempdir2.path().join("platune.db");
        let db2 = Database::connect(sql_path2, true).await;
        db2.migrate().await;

        let mut manager2 = Manager::new(&db2, &config);
        manager2.delim = r"\";
        manager2.validate_paths = false;

        manager2.add_folder(r"C:\test").await;
        manager2.register_drive(r"C:\").await.unwrap();

        let folders = manager2.get_all_folders().await;

        db.close().await;
        db2.close().await;
        tempdir.close().unwrap();
        tempdir2.close().unwrap();

        assert_eq!(vec![r"C:\test\"], folders);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_validate_path() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut manager) = setup(&tempdir).await;

        manager.delim = r"\";

        let res = manager.register_drive(r"/some/invalid/path").await;

        db.close().await;
        tempdir.close().unwrap();

        assert!(res.is_err());
    }

    async fn setup(tempdir: &TempDir) -> (Database, Manager) {
        let sql_path = tempdir.path().join("platune.db");
        let config_path = tempdir.path().join("platuneconfig");
        let db = Database::connect(sql_path, true).await;
        db.migrate().await;
        let config = Config::new_from_path(config_path);
        let manager = Manager::new(&db, &config);
        (db, manager)
    }
}
