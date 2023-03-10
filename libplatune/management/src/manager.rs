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
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
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
    config: Arc<Box<dyn Config + Send + Sync>>,
    validate_paths: bool,
    delim: &'static str,
}

impl std::ops::Deref for Manager {
    type Target = Database;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl Manager {
    pub fn new(db: &Database, config: Arc<Box<dyn Config + Send + Sync>>) -> Self {
        Self {
            db: db.clone(),
            config,
            validate_paths: true,
            delim: if cfg!(windows) { r"\" } else { "/" },
        }
    }

    pub async fn register_drive<P: AsRef<Path>>(&self, path: P) -> Result<(), ManagerError> {
        let path = path.as_ref();
        if self.validate_paths && !path.exists() {
            return Err(ManagerError::InvalidPath(path.to_owned()));
        }

        let path = self.clean_path(&path.to_string_lossy());

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

    pub async fn sync(&mut self, paths: Option<Vec<String>>) -> Result<ProgressStream, DbError> {
        let folders = match paths {
            Some(paths) => paths,
            None => self.get_all_folders().await?,
        };

        let mount = self.get_registered_mount().await;
        Ok(self.db.sync(folders, mount).await)
    }

    async fn replace_prefix(&self, paths: Vec<&str>) -> Vec<String> {
        let paths = paths.into_iter().map(|new_path| self.clean_path(new_path));

        match self.get_registered_mount().await {
            Some(mount) => paths
                .map(|path| match path.find(&mount[..]) {
                    Some(0) => path.replacen(&mount[..], "", 1),
                    _ => path.to_owned(),
                })
                .collect::<Vec<_>>(),
            None => paths.collect(),
        }
    }

    pub async fn expand_paths(&self, folders: Vec<String>) -> Vec<String> {
        let res = match self.get_registered_mount().await {
            Some(mount) => folders.into_iter().map(|f| format!("{mount}{f}")).collect(),
            None => folders,
        };
        res.into_iter()
            .map(|r| r.replace('/', self.delim))
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
        correlation_ids: Vec<i64>,
        entry_type: EntryType,
    ) -> Result<Vec<LookupEntry>, DbError> {
        let mut res = self.db.lookup(correlation_ids, entry_type).await?;
        self.update_paths(&mut res).await;

        Ok(res)
    }

    pub async fn rename_path<P>(&mut self, from: P, to: P) -> Result<(), DbError>
    where
        P: AsRef<Path>,
    {
        let mount = self.get_registered_mount().await;
        let from = clean_file_path(&from, &mount);
        let to = clean_file_path(&to, &mount);
        self.db.rename_path(from, to).await?;
        Ok(())
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

    async fn update_paths<T>(&self, paths: &mut [T])
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
        self.db.search(query, options).await
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
            .map_err(|e| ManagerError::WriteError(format!("{e:?}")))
    }
}

#[cfg(test)]
#[path = "./manager_test.rs"]
mod manager_test;
