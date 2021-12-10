pub use crate::entry_type::EntryType;
pub use crate::search::search_options::SearchOptions;
pub use crate::search::search_result::SearchResult;
use crate::{
    config::Config,
    database::{Database, DeletedEntry, LookupEntry},
    db_error::DbError,
    path_util::{clean_file_path, update_path, PathMut},
    sync::progress_stream::ProgressStream,
};
use regex::Regex;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("{0} is not a valid path")]
    InvalidPath(PathBuf),
    #[error("Error writing file: {0}")]
    WriteError(String),
    #[error(transparent)]
    DbError(DbError),
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

    pub async fn register_drive<P: AsRef<Path>>(&self, path: P) -> Result<(), ManagerError> {
        let path = path.as_ref();
        if self.validate_paths && !path.exists() {
            return Err(ManagerError::InvalidPath(path.to_owned()));
        }

        let path = self.clean_path(&path.to_string_lossy().into_owned());

        match self.config.get_drive_id() {
            Some(drive_id) => {
                let rows = self
                    .db
                    .update_mount(drive_id, &path)
                    .await
                    .map_err(ManagerError::DbError)?;
                if rows == 0 {
                    self.set_drive(&path).await?;
                }
            }
            None => {
                self.set_drive(&path).await?;
            }
        };

        let folders = self
            .db
            .get_all_folders()
            .await
            .map_err(ManagerError::DbError)?;
        for folder in folders {
            let new_folder = folder.replacen(&path, "", 1);
            self.db
                .update_folder(folder, new_folder)
                .await
                .map_err(ManagerError::DbError)?;
        }

        Ok(())
    }

    pub async fn add_folder(&self, path: &str) -> Result<(), DbError> {
        self.add_folders(vec![path]).await
    }

    pub async fn add_folders(&self, paths: Vec<&str>) -> Result<(), DbError> {
        let new_paths = self.replace_prefix(paths).await;
        self.db.add_folders(new_paths).await
    }

    pub async fn get_all_folders(&self) -> Result<Vec<String>, DbError> {
        let folders = self.db.get_all_folders().await?;
        Ok(self.expand_paths(folders).await)
    }

    pub async fn sync(&mut self) -> Result<ProgressStream, DbError> {
        let folders = self.get_all_folders().await?;
        let mount = self.get_registered_mount().await;
        Ok(self.db.sync(folders, mount).await)
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
            None => paths.collect(),
        };

        new_paths
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
    ) -> Result<Vec<LookupEntry>, DbError> {
        let mut res = self.db.lookup(correlation_ids, entry_type).await?;
        self.update_paths(&mut res).await;

        Ok(res)
    }

    pub async fn get_song_by_path<P>(&self, path: P) -> Result<Option<LookupEntry>, DbError>
    where
        P: AsRef<Path>,
    {
        let mount = self.get_registered_mount().await;
        let path = clean_file_path(&path, &mount);

        let res = self.db.get_song_by_path(path).await?;
        match res {
            Some(mut res) => {
                self.update_path(&mut res).await;
                Ok(Some(res))
            }
            None => Ok(None),
        }
    }

    async fn update_paths<T>(&self, paths: &mut Vec<T>)
    where
        T: PathMut,
    {
        if let Some(mount) = self.get_registered_mount().await {
            for entry in paths.iter_mut() {
                update_path(entry, &mount);
            }
        }
    }

    async fn update_path<T>(&self, path: &mut T)
    where
        T: PathMut,
    {
        if let Some(mount) = self.get_registered_mount().await {
            update_path(path, &mount);
        }
    }

    pub async fn search(
        &self,
        query: &str,
        options: SearchOptions<'_>,
    ) -> Result<Vec<SearchResult>, DbError> {
        Ok(self.db.search(query, options).await?)
    }

    pub async fn get_deleted_songs(&self) -> Result<Vec<DeletedEntry>, DbError> {
        let mut deleted = self.db.get_deleted_songs().await?;
        self.update_paths(&mut deleted).await;
        Ok(deleted)
    }

    pub async fn delete_tracks(&self, ids: Vec<i64>) -> Result<(), DbError> {
        self.db.delete_tracks(ids).await
    }

    fn clean_path(&self, path: &str) -> String {
        let mut path = path.replace(self.delim, "/");
        let re = Regex::new(r"/+").unwrap();
        path = re.replace_all(&path, "/").to_string();
        if !path.ends_with('/') {
            path += "/";
        }
        path
    }

    async fn set_drive(&self, path: &str) -> Result<(), ManagerError> {
        let id = self
            .db
            .add_mount(path)
            .await
            .map_err(ManagerError::DbError)?;

        self.config
            .set_drive_id(id)
            .map_err(|e| ManagerError::WriteError(format!("{:?}", e)))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{config::Config, database::Database};
    use tempfile::TempDir;
    use tokio::time::timeout;

    use super::Manager;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_add_folders() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut config) = setup(&tempdir).await;
        config.delim = r"\";

        config.add_folder(r"test1\").await.unwrap();
        config.add_folder("test1").await.unwrap();
        config.add_folder("test2").await.unwrap();
        config.add_folder(r"test2\\").await.unwrap();
        let folders = config.get_all_folders().await.unwrap();

        timeout(Duration::from_secs(5), db.close())
            .await
            .unwrap_or_default();

        assert_eq!(vec![r"test1\", r"test2\"], folders);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_change_mount() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut manager) = setup(&tempdir).await;
        manager.delim = r"\";
        manager.validate_paths = false;

        manager.register_drive(r"C:\\").await.unwrap();
        manager.add_folder(r"C:\test").await.unwrap();
        manager.add_folder(r"C:\\test\\").await.unwrap();
        let folders1 = manager.get_all_folders().await.unwrap();
        manager.register_drive(r"D:\").await.unwrap();
        let folders2 = manager.get_all_folders().await.unwrap();

        timeout(Duration::from_secs(5), db.close())
            .await
            .unwrap_or_default();

        assert_eq!(vec![r"C:\test\"], folders1);
        assert_eq!(vec![r"D:\test\"], folders2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_change_mount_after() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut manager) = setup(&tempdir).await;
        manager.delim = r"\";
        manager.validate_paths = false;

        manager.add_folder(r"C:\test").await.unwrap();
        manager.add_folder(r"C:\\test\\").await.unwrap();
        manager.register_drive(r"C:\").await.unwrap();
        let folders1 = manager.get_all_folders().await.unwrap();
        manager.register_drive(r"D:\").await.unwrap();
        let folders2 = manager.get_all_folders().await.unwrap();

        timeout(Duration::from_secs(5), db.close())
            .await
            .unwrap_or_default();

        assert_eq!(vec![r"C:\test\"], folders1);
        assert_eq!(vec![r"D:\test\"], folders2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_multiple_mounts() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut manager) = setup(&tempdir).await;
        let config_path2 = tempdir.path().join("platuneconfig2");
        let config2 = Config::new_from_path(config_path2).unwrap();
        let mut manager2 = Manager::new(&db, &config2);
        manager.delim = r"\";
        manager.validate_paths = false;
        manager2.delim = r"\";
        manager2.validate_paths = false;

        manager.add_folder(r"C:\test").await.unwrap();
        manager.add_folder(r"C:\\test\\").await.unwrap();
        manager.register_drive(r"C:\").await.unwrap();
        let folders1 = manager.get_all_folders().await.unwrap();
        manager2.register_drive(r"D:\").await.unwrap();
        let folders2 = manager2.get_all_folders().await.unwrap();

        timeout(Duration::from_secs(5), db.close())
            .await
            .unwrap_or_default();

        assert_eq!(vec![r"C:\test\"], folders1);
        assert_eq!(vec![r"D:\test\"], folders2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_reset_drive_id_if_missing() {
        let tempdir = TempDir::new().unwrap();
        let sql_path = tempdir.path().join("platune.db");
        let config_path = tempdir.path().join("platuneconfig");
        let db = Database::connect(sql_path, true).await.unwrap();
        db.migrate().await.unwrap();
        let config = Config::new_from_path(config_path.clone()).unwrap();
        let mut manager = Manager::new(&db, &config);
        manager.delim = r"\";
        manager.validate_paths = false;

        manager.add_folder(r"C:\test").await.unwrap();
        manager.register_drive(r"C:\").await.unwrap();

        let tempdir2 = TempDir::new().unwrap();
        let sql_path2 = tempdir2.path().join("platune.db");
        let db2 = Database::connect(sql_path2, true).await.unwrap();
        db2.migrate().await.unwrap();

        let mut manager2 = Manager::new(&db2, &config);
        manager2.delim = r"\";
        manager2.validate_paths = false;

        manager2.add_folder(r"C:\test").await.unwrap();
        manager2.register_drive(r"C:\").await.unwrap();

        let folders = manager2.get_all_folders().await.unwrap();

        timeout(Duration::from_secs(5), db.close())
            .await
            .unwrap_or_default();
        timeout(Duration::from_secs(5), db2.close())
            .await
            .unwrap_or_default();

        assert_eq!(vec![r"C:\test\"], folders);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_validate_path() {
        let tempdir = TempDir::new().unwrap();
        let (db, mut manager) = setup(&tempdir).await;

        manager.delim = r"\";

        let res = manager.register_drive(r"/some/invalid/path").await;

        timeout(Duration::from_secs(5), db.close())
            .await
            .unwrap_or_default();

        assert!(res.is_err());
    }

    async fn setup(tempdir: &TempDir) -> (Database, Manager) {
        let sql_path = tempdir.path().join("platune.db");
        let config_path = tempdir.path().join("platuneconfig");
        let db = Database::connect(sql_path, true).await.unwrap();
        db.migrate().await.unwrap();
        let config = Config::new_from_path(config_path).unwrap();
        let manager = Manager::new(&db, &config);
        (db, manager)
    }
}
