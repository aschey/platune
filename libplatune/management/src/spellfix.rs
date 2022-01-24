use rusqlite::LoadExtensionGuard;
use sqlx::{pool::PoolConnection, Pool, Sqlite, SqliteConnection};
use std::{env::var, path::Path};

use crate::db_error::DbError;

pub(crate) async fn acquire_with_spellfix(
    pool: &Pool<Sqlite>,
) -> Result<PoolConnection<Sqlite>, DbError> {
    let mut con = pool
        .acquire()
        .await
        .map_err(|e| DbError::DbError(format!("{:?}", e)))?;
    load_spellfix(&mut con).await?;
    Ok(con)
}

pub(crate) async fn load_spellfix(con: &mut SqliteConnection) -> Result<(), DbError> {
    let spellfix_lib = match var("SPELLFIX_LIB") {
        Ok(res) => res,
        #[cfg(target_os = "linux")]
        Err(_) => "./assets/linux/spellfix.o".to_owned(),
        #[cfg(target_os = "windows")]
        Err(_) => "./assets/windows/spellfix.dll".to_owned(),
    };
    load_extension(con, &spellfix_lib)
        .await
        .map_err(DbError::SpellfixLoadError)
}

async fn load_extension<P: AsRef<Path>>(
    con: &mut SqliteConnection,
    dylib_path: &P,
) -> Result<(), String> {
    let handle = con
        .lock_handle()
        .await
        .map_err(|e| format!("{:?}", e))?
        .as_raw_handle();

    // Safety: we shouldn't run any untrusted queries while the extension guard is active
    unsafe {
        let rusqlite_con =
            rusqlite::Connection::from_handle(handle.as_ptr()).map_err(|e| format!("{:?}", e))?;
        let _guard = LoadExtensionGuard::new(&rusqlite_con).map_err(|e| format!("{:?}", e))?;

        rusqlite_con
            .load_extension(dylib_path, None)
            .map_err(|e| format!("{:?}", e))?;
    }

    Ok(())
}
