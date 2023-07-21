pub mod config;
mod consts;
pub mod database;
pub mod db_error;
pub mod entry_type;
pub mod file_watch_manager;
pub mod manager;
mod path_util;
pub mod search;
mod sql_util;
pub mod sync;

#[cfg(feature = "ffi")]
uniffi::include_scaffolding!("management");
