use libsqlite3_sys::{sqlite3, sqlite3_load_extension};
use sqlx::{pool::PoolConnection, Pool, Sqlite, SqliteConnection};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    path::Path,
    ptr,
};

use crate::db_error::DbError;

pub(crate) async fn acquire_with_spellfix(
    pool: &Pool<Sqlite>,
) -> Result<PoolConnection<Sqlite>, DbError> {
    let mut con = pool
        .acquire()
        .await
        .map_err(|e| DbError::DbError(e.to_string()))?;
    load_spellfix(&mut con)?;
    Ok(con)
}

pub(crate) fn load_spellfix(con: &mut SqliteConnection) -> Result<(), DbError> {
    let handle = con.as_raw_handle();
    let spellfix_lib = match std::env::var("SPELLFIX_LIB") {
        Ok(res) => res,
        #[cfg(target_os = "linux")]
        Err(_) => "./assets/linux/spellfix.o".to_owned(),
        #[cfg(target_os = "windows")]
        Err(_) => "./assets/windows/spellfix.dll".to_owned(),
    };

    load_extension(handle, &spellfix_lib).map_err(DbError::SpellfixLoadError)
}

#[cfg(not(unix))]
fn path_to_cstring<P: AsRef<Path>>(p: &P) -> CString {
    let s = p.as_ref().to_string_lossy().to_string();
    CString::new(s).unwrap()
}

#[cfg(unix)]
fn path_to_cstring<P: AsRef<Path>>(p: &P) -> CString {
    use std::os::unix::ffi::OsStrExt;
    CString::new(p.as_ref().as_os_str().as_bytes()).unwrap()
}

unsafe fn errmsg_to_string(errmsg: *const c_char) -> String {
    let c_slice = CStr::from_ptr(errmsg).to_bytes();
    String::from_utf8_lossy(c_slice).into_owned()
}

fn load_extension<P: AsRef<Path>>(db: *mut sqlite3, dylib_path: &P) -> Result<(), String> {
    let dylib_str = path_to_cstring(dylib_path);
    unsafe {
        let mut errmsg: *mut c_char = ptr::null_mut();

        let res = sqlite3_load_extension(db, dylib_str.as_ptr(), ptr::null(), &mut errmsg);
        if res != 0 {
            return Err(errmsg_to_string(errmsg));
        }

        Ok(())
    }
}
