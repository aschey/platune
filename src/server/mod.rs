mod models;
use crate::schema;
use crate::schema::album::dsl::*;
use crate::schema::album_artist::dsl::*;
use crate::schema::artist::dsl::*;
use crate::schema::folder::dsl::*;
use crate::schema::import_temp::dsl::*;
use crate::schema::mount::dsl::*;
use crate::schema::song::dsl::*;
use crate::schema::tag::dsl::*;
use actix_cors::Cors;
use actix_files as fs;
use actix_http::http::header::{HeaderName, HeaderValue};
use actix_service::Service;
use actix_web::{
    body,
    dev::Server,
    error, get,
    http::StatusCode,
    http::{header, Method},
    middleware,
    web::Query,
    App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use anyhow::Result as AnyResult;
use async_std::prelude::*;
use async_std::task;
use diesel;
use diesel::prelude::*;
use diesel::sql_types::*;
use diesel::sqlite::SqliteConnection;
use dirs::home_dir;
use dotenv::dotenv;
use failure::Fail;
use fstrings::*;
use futures::future::FutureExt;
use futures::join;
use image::{imageops, GenericImageView, ImageBuffer, RgbImage};
use itertools::Itertools;
use models::{folder::*, import::*, mount::*};
use paperclip::actix::{
    api_v2_errors,
    api_v2_operation,
    // use this instead of actix_web::web
    web::{self, Json, Path},
    Apiv2Schema,
    // extension trait for actix_web::App and proc-macro attributes
    OpenApiExt,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fs::{copy, read_dir, remove_file, DirEntry, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;
use std::{env, vec::Vec};
use std::{str, sync::mpsc};
use sysinfo::{DiskExt, SystemExt};

embed_migrations!();

const IS_WINDOWS: bool = cfg!(windows);
const DATABASE_URL: &str = "DATABASE_URL";
fn get_delim() -> &'static str {
    return if IS_WINDOWS { "\\" } else { "/" };
}

fn convert_delim(path: String) -> String {
    return if IS_WINDOWS {
        path.replace("\\", "/")
    } else {
        path.replace("/", "\\")
    };
}

fn convert_delim_windows(path: String) -> String {
    return path.replace("/", "\\");
}

fn convert_delim_unix(path: String) -> String {
    return path.replace("\\", "/");
}

fn get_delim_escaped() -> &'static str {
    return if IS_WINDOWS { "\\\\" } else { "/" };
}

#[cfg(windows)]
fn get_path() -> schema::folder::full_path_windows {
    full_path_windows
}
#[cfg(unix)]
fn get_path() -> schema::folder::full_path_unix {
    full_path_unix
}

#[cfg(windows)]
fn get_mount_path() -> schema::mount::windows_path {
    windows_path
}
#[cfg(unix)]
fn get_mount_path() -> schema::mount::unix_path {
    unix_path
}

#[cfg(windows)]
fn get_song_path() -> schema::song::song_path_windows {
    song_path_windows
}
#[cfg(unix)]
fn get_song_path() -> schema::song::song_path_unix {
    song_path_unix
}

#[derive(RustEmbed)]
#[folder = "src/server/swagger"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => {
            let body: body::Body = match content {
                Cow::Borrowed(bytes) => bytes.into(),
                Cow::Owned(bytes) => bytes.into(),
            };
            HttpResponse::Ok()
                .content_type(mime_guess::from_path(path).first_or_octet_stream().as_ref())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

fn index() -> HttpResponse {
    handle_embedded_file("index.html")
}

pub fn establish_connection() -> AnyResult<SqliteConnection> {
    dotenv::from_path(get_env_path()).ok();
    let database_url = env::var(DATABASE_URL)?;
    Ok(SqliteConnection::establish(&database_url)?)
}

pub fn run_server(tx: mpsc::Sender<Server>) -> std::io::Result<()> {
    if !get_env_path().exists() {
        write_env(
            &std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
        );
    }

    let mut sys = actix_rt::System::new("server");

    let srv = HttpServer::new(|| {
        let mut builder = App::new()
            .wrap(Cors::new().finish())
            .wrap_api()
            // REST endpoints
            .route("/dirsInit", web::get().to(get_dirs_init))
            .route("/dirs", web::get().to(get_dirs))
            .route("/configuredFolders", web::get().to(get_configured_folders))
            .route("/isWindows", web::get().to(get_is_windows))
            .route("/updateFolders", web::put().to(update_folders))
            .route("/getDbPath", web::get().to(get_db_path))
            .route("/updateDbPath", web::put().to(update_db_path))
            .route("/getNtfsMounts", web::get().to(get_ntfs_mounts))
            .route("/updatePathMappings", web::put().to(update_path_mappings))
            .route("/songs", web::get().to(get_songs))
            .route("/albumArt", web::get().to(get_album_art))
            .route("/albumArtColors", web::get().to(get_art_colors))
            .route("/search", web::get().to(search))
            .route("/sync", web::put().to(get_all_files))
            .route("/tags", web::post().to(add_tag))
            .route("/tags/{tag_id}", web::put().to(update_tag))
            .route("/tags", web::get().to(get_tags))
            .route("/tags/{tag_id}", web::delete().to(delete_tag))
            .with_json_spec_at("/spec")
            .build()
            // static files
            .wrap_fn(|req, srv| {
                let path = req.path().to_owned();
                srv.call(req).map(move |res| {
                    if path == "/index.html" || path == "/" {
                        match res {
                            Ok(mut r) => {
                                r.headers_mut().insert(
                                    HeaderName::try_from("Cache-Control").unwrap(),
                                    HeaderValue::try_from("no-cache").unwrap(),
                                );
                                Ok(r)
                            }
                            Err(r) => Err(r),
                        }
                    } else {
                        res
                    }
                })
            })
            .service(web::resource("/swagger").route(web::get().to(index)));

        let connection_res = establish_connection();
        if let Ok(connection) = connection_res {
            embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();
            let paths = folder
                .select(get_path())
                .load::<String>(&connection)
                .unwrap();
            for path in paths {
                builder = builder.service(
                    fs::Files::new(&to_url_path(path.to_owned()), path.to_owned())
                        .show_files_listing(),
                );
            }
        }

        let app = builder
            // Paths are matched in order so this needs to be last
            .service(fs::Files::new("/", "./src/ui/platune/build").index_file("index.html"));
        return app;
    })
    .bind("0.0.0.0:5000")?
    .run();

    // send server controller to main thread
    let _ = tx.send(srv.clone());

    // run future
    sys.block_on(srv)
}

fn to_url_path(drive_path: String) -> String {
    if !IS_WINDOWS {
        return drive_path;
    }
    let replaced = convert_delim_unix(drive_path.replace(":", ""));
    return f!("/{replaced}");
}

fn get_timestamp(time: SystemTime) -> i32 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32
}

fn get_all_files_rec(start_path: String, original: String, other: String) -> Vec<NewImport> {
    let mut all_files = Vec::<NewImport>::new();
    let mut dirs = std::fs::read_dir(start_path).unwrap();
    let original2 = original.clone();
    let other2 = other.clone();
    while let Some(dir_res) = dirs.next() {
        let dir = dir_res.unwrap();
        let path = dir.path();
        let full_path = path.to_str().unwrap();
        if path.is_file() {
            if full_path.ends_with(".mp3") || full_path.ends_with(".m4a") {
                let f = katatsuki::Track::from_path(std::path::Path::new(full_path), None).unwrap();
                let mut n = NewImport {
                    import_artist: f.artist.trim().to_owned(),
                    import_album: f.album.trim().to_owned(),
                    import_album_artist: if f.album_artists.len() > 0 && f.album_artists[0] != "" {
                        f.album_artists[0].trim().to_owned()
                    } else {
                        f.artist.trim().to_owned()
                    },
                    import_song_path_windows: "".to_string(),
                    import_song_path_unix: "".to_string(),
                    import_title: f.title.trim().to_owned(),
                    import_track_number: f.track_number,
                    import_disc_number: if f.disc_number > 0 { f.disc_number } else { 1 },
                    import_year: f.year,
                    import_duration: f.duration,
                    import_sample_rate: f.sample_rate,
                    import_bit_rate: f.bitrate,
                    import_album_art: read_album_art(f.album_art, path.to_owned()),
                };
                if IS_WINDOWS {
                    n.import_song_path_windows = full_path.to_owned();
                    if other2 != "" {
                        n.import_song_path_unix =
                            convert_delim(full_path.to_owned().replace(&original2, &other2));
                    }
                } else {
                    n.import_song_path_unix = full_path.to_owned();
                    if other2 != "" {
                        n.import_song_path_windows =
                            convert_delim(full_path.to_owned().replace(&original2, &other2));
                    }
                }
                all_files.push(n);
            }
        } else {
            let inner_files =
                get_all_files_rec(full_path.to_owned(), original2.clone(), other2.clone());
            all_files = [all_files, inner_files].concat();
        }
    }
    return all_files;
}

fn read_album_art(from_tag: Vec<u8>, path: PathBuf) -> Option<Vec<u8>> {
    if from_tag.len() > 0 {
        return Some(from_tag);
    } else {
        let parent = path.parent().unwrap();
        let mut parent_files = std::fs::read_dir(parent).unwrap();
        let cover_opt = parent_files.find(|f| {
            let temp = f.as_ref().unwrap().path();
            let path = temp.to_str().unwrap();
            return path.to_lowercase().ends_with("cover.jpg")
                || path.to_lowercase().ends_with("cover.png");
        });
        if let Some(cover) = cover_opt {
            let img = std::fs::read(cover.unwrap().path()).unwrap();
            return Some(img);
        }
        return None;
    }
}

async fn get_all_files_parallel(root: String, other: String) {
    let proc_count = num_cpus::get() as f32;
    let dirs_and_files = read_dir(root.to_owned()).unwrap();

    let dirs = dirs_and_files
        .filter_map(|d| {
            let p = d.as_ref().unwrap().path();
            if p.is_dir() {
                Some(p.to_str().unwrap().to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let dir_count = dirs.len() as f32;
    let dirs_per_thread = (dir_count / proc_count).ceil() as usize;
    let mut handles = Vec::<_>::new();
    for i in 0..proc_count as usize {
        let i = i.clone();
        let dirs = dirs.clone();
        let root = root.clone();
        let other = other.clone();
        handles.push(task::spawn(async move {
            let mut files = Vec::<_>::new();
            for path in dirs
                [dirs_per_thread * i..std::cmp::min(dirs.len(), dirs_per_thread * (i + 1))]
                .to_vec()
            {
                files = [
                    files,
                    get_all_files_rec(path, root.to_owned(), other.to_owned()),
                ]
                .concat();
            }
            return files;
        }));
    }
    let all = futures::future::join_all(handles).await;
    let res = all.iter().flatten().collect::<Vec<_>>();

    let connection = establish_connection().unwrap();
    let batch_size = 5000;
    let batches = (res.len() as f32 / batch_size as f32).ceil() as usize;
    let _ = diesel::delete(import_temp).execute(&connection);
    for i in 0..batches {
        connection
            .transaction::<_, diesel::result::Error, _>(|| {
                let batch =
                    res[i * batch_size..std::cmp::min(res.len(), (i + 1) * batch_size)].to_vec();
                for row in batch {
                    let _r = diesel::insert_into(import_temp)
                        .values(row)
                        .execute(&connection)
                        .unwrap();
                }
                Ok(())
            })
            .unwrap();
    }

    let _ = diesel::sql_query(
        "
        insert into artist(artist_name)
        select import_artist from import_temp
        left outer join artist on artist_name = import_artist
        where artist_name is null
        group by import_artist
        ",
    )
    .execute(&connection)
    .unwrap();

    let _ = diesel::sql_query(
        "
        insert into album_artist(album_artist_name)
        select import_album_artist from import_temp
        left outer join album_artist on album_artist_name = import_album_artist
        where album_artist_name is null
        group by import_album_artist
        ",
    )
    .execute(&connection)
    .unwrap();

    let _ = diesel::sql_query(
        "
        insert into album(album_name, album_year, album_month, album_day, album_artist_id)
        select import_album, case when count(distinct import_year) > 1 then 0 else import_year end, 0, 0,  album_artist.album_artist_id
        from import_temp
        inner join album_artist on album_artist_name = import_album_artist
        left outer join album on album_name = import_album and album.album_artist_id = album_artist.album_artist_id
        where album_name is null and album.album_artist_id is null
        group by import_album_artist, import_album
        "
    ).execute(&connection).unwrap();
    let _ = diesel::sql_query(
        "
        insert into song(song_path_unix, song_path_windows, metadata_modified_date, artist_id, song_title, 
            album_id, track_number, play_count, disc_number, song_year, song_month, song_day, duration,
            sample_rate, bit_rate, album_art, is_deleted)
        select import_song_path_unix, import_song_path_windows, strftime('%s', 'now'), artist.artist_id, import_title, album.album_id, import_track_number, 0, 
            import_disc_number, import_year, 0, 0, import_duration, import_sample_rate, import_bit_rate, import_album_art, false
        from import_temp
        inner join artist on artist_name = import_artist
        inner join album_artist on album_artist_name = import_album_artist
        inner join album on album_name = import_album and album.album_artist_id = album_artist.album_artist_id
        left outer join song on song_path_unix = import_song_path_unix or song_path_windows = import_song_path_windows
        where song.song_path_unix is null or song.song_path_windows is null
        "
    ).execute(&connection).unwrap();
    let _ = diesel::delete(import_temp).execute(&connection);
}

#[api_v2_operation]
pub async fn get_all_files() -> Result<Json<()>, ()> {
    let t = actix_rt::spawn(async {
        let now = std::time::Instant::now();
        let connection = establish_connection().unwrap();
        let dirs: Vec<_>;
        if IS_WINDOWS {
            dirs = folder
                .select((full_path_windows, full_path_unix))
                .load::<(String, String)>(&connection)
                .unwrap();
        } else {
            dirs = folder
                .select((full_path_unix, full_path_windows))
                .load::<(String, String)>(&connection)
                .unwrap();
        }
        for dir in dirs {
            get_all_files_parallel(dir.0.to_owned(), dir.1.to_owned()).await;
        }
        println!("{}", now.elapsed().as_secs());
    });
    return Ok(Json(()));
}

fn filter_dirs(res: Result<DirEntry, std::io::Error>, delim: &str) -> Option<Dir> {
    let path = res.unwrap().path();
    let str_path = String::from(path.to_str().unwrap());
    let dir_name = String::from(str_path.split(delim).last().unwrap());
    if !dir_name.starts_with(".") {
        Some(Dir {
            name: dir_name,
            is_file: path.is_file(),
        })
    } else {
        None
    }
}

pub trait StrVecExt {
    fn sort_case_insensitive(&mut self);
}

impl StrVecExt for Vec<String> {
    fn sort_case_insensitive(&mut self) {
        &self.sort_by(|l, r| Ord::cmp(&l.to_lowercase(), &r.to_lowercase()));
    }
}

pub trait DirVecExt {
    fn sort_case_insensitive(&mut self);
}

impl DirVecExt for Vec<Dir> {
    fn sort_case_insensitive(&mut self) {
        &self.sort_by(|l, r| Ord::cmp(&l.name.to_lowercase(), &r.name.to_lowercase()));
    }
}

pub trait SongExt {
    fn sort_case_insensitive(&mut self, field: String);
}

impl SongExt for Vec<Song> {
    fn sort_case_insensitive(&mut self, field: String) {
        &self.sort_by(|l, r| Ord::cmp(&l.track, &r.track));
        &self.sort_by(|l, r| Ord::cmp(&l.disc, &r.disc));
        &self.sort_by(|l, r| Ord::cmp(&l.album.to_lowercase(), &r.album.to_lowercase()));
        &self.sort_by(|l, r| Ord::cmp(&l.artist.to_lowercase(), &r.artist.to_lowercase()));
        &self.sort_by(|l, r| {
            Ord::cmp(
                &l.album_artist.to_lowercase(),
                &r.album_artist.to_lowercase(),
            )
        });
    }
}

#[api_v2_errors(code = 400, code = 500)]
#[derive(Fail, Debug)]
#[fail(display = "named error")]
struct HttpError {
    result: String,
}

// Use default implementation for `error_response()` method
impl error::ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        let mut resp = HttpResponse::new(self.status_code());
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        resp.set_body(body::Body::from(self.result.to_owned()))
    }
}

fn get_dir_name(disk: &std::path::Path) -> String {
    let mut str_path = String::from(disk.to_str().unwrap());
    if IS_WINDOWS {
        str_path = str_path.replace("\\", "");
    }
    return str_path;
}

#[api_v2_operation]
async fn get_dirs_init() -> Result<Json<DirResponse>, ()> {
    let system = sysinfo::System::new_all();
    let disks = system
        .get_disks()
        .iter()
        .map(|d| Dir {
            is_file: false,
            name: get_dir_name(d.get_mount_point()),
        })
        .collect::<Vec<_>>();
    return Ok(Json(DirResponse { dirs: disks }));
}

#[api_v2_operation]
async fn get_dirs(dir_request_query: Query<DirRequest>) -> Result<Json<DirResponse>, ()> {
    let dir_request = dir_request_query.into_inner();
    let mut entries = read_dir(dir_request.dir.as_str())
        .unwrap()
        .filter_map(|res| filter_dirs(res, get_delim()))
        .collect::<Vec<_>>();

    entries.sort_case_insensitive();
    let response = Json(DirResponse { dirs: entries });
    return Ok(response);
}

#[api_v2_operation]
async fn get_is_windows() -> Result<Json<bool>, ()> {
    return Ok(Json(IS_WINDOWS));
}

fn get_configured_folders_helper() -> Vec<String> {
    let connection = establish_connection().unwrap();
    let results = folder.load::<Folder>(&connection).expect("error");
    let paths = results
        .iter()
        .map(|rr| get_platform_folder(rr).clone())
        .filter(|r| r.len() > 0)
        .collect();
    return paths;
}

#[api_v2_operation]
async fn get_configured_folders() -> Result<Json<Vec<String>>, ()> {
    let paths = get_configured_folders_helper();
    return Ok(Json(paths));
}

fn get_ntfs_mounts_helper() -> Vec<String> {
    let system = sysinfo::System::new_all();
    let disks = system.get_disks();
    let fuse_disks = disks
        .iter()
        .filter(|d| str::from_utf8(d.get_file_system()).unwrap() == "fuseblk")
        .map(|d| get_dir_name(d.get_mount_point()))
        .collect::<Vec<_>>();
    let configured = get_configured_folders_helper();
    let configured_fuse = fuse_disks
        .into_iter()
        .filter(|f| configured.iter().any(|c| c.starts_with(f)))
        .collect::<Vec<_>>();
    return configured_fuse;
}

#[api_v2_operation]
async fn get_ntfs_mounts() -> Result<Json<Vec<NtfsMapping>>, ()> {
    let connection = establish_connection().unwrap();
    let fs_fuse = get_ntfs_mounts_helper();
    let mapped = mount
        .select((unix_path, windows_path))
        .load::<(String, String)>(&connection)
        .unwrap();
    if IS_WINDOWS {
        let all = mapped
            .iter()
            .map(|m| NtfsMapping {
                dir: m.0.to_owned(),
                drive: m.1.to_owned(),
            })
            .collect::<Vec<_>>();
        return Ok(Json(all));
    }
    let mapped_unix = mapped.iter().map(|m| m.0.to_owned()).collect::<Vec<_>>();
    let mut mappings = fs_fuse
        .iter()
        .filter(|f| mapped_unix.contains(f))
        .map(|f| NtfsMapping {
            dir: f.to_owned(),
            drive: mapped
                .iter()
                .filter(|m| m.0 == f.to_owned())
                .map(|m| m.1.to_owned())
                .collect(),
        })
        .collect::<Vec<_>>();
    mappings.extend(
        fs_fuse
            .iter()
            .filter(|f| !mapped_unix.contains(f))
            .map(|f| NtfsMapping {
                dir: f.to_owned(),
                drive: "".to_owned(),
            })
            .collect::<Vec<_>>(),
    );
    return Ok(Json(mappings));
}

fn get_subfolders(new_folders: Vec<String>) -> Vec<String> {
    let copy = new_folders.to_vec();
    let dedup = &new_folders
        .into_iter()
        .dedup_by(|l, r| r.starts_with(l))
        .collect::<Vec<_>>();

    let lala = copy
        .into_iter()
        .filter(|f| !dedup.contains(f))
        .collect::<Vec<_>>();
    return lala;
}

fn get_dupe_folders(new_folders: Vec<String>) -> Vec<(String, Vec<String>)> {
    let grouped = new_folders
        .into_iter()
        .group_by(|f| String::from(f))
        .into_iter()
        .map(|(key, group)| (key, group.collect::<Vec<_>>()))
        .collect::<Vec<(String, Vec<String>)>>();
    return grouped;
}

fn get_platform_folder(f: &Folder) -> String {
    if IS_WINDOWS {
        f.full_path_windows.to_owned()
    } else {
        f.full_path_unix.to_owned()
    }
}

fn new_folder(path: String) -> NewFolder {
    if IS_WINDOWS {
        NewFolder {
            full_path_unix: "".to_owned(),
            full_path_windows: path,
        }
    } else {
        NewFolder {
            full_path_unix: path,
            full_path_windows: "".to_owned(),
        }
    }
}

#[api_v2_operation]
async fn update_folders(new_folders_json: Json<FolderUpdate>) -> Result<Json<()>, HttpError> {
    let new_folders_req = new_folders_json.into_inner();
    let mut new_folders = new_folders_req.folders.to_vec();
    new_folders.sort_case_insensitive();
    let new_folders3 = new_folders.to_vec();
    let grouped = get_dupe_folders(new_folders);
    for (_, group) in grouped.into_iter() {
        if group.len() > 1 {
            let dup = group[0].to_owned();
            return Err(HttpError {
                result: f!("Duplicate folder chosen: {dup}"),
            });
        }
    }

    let invalid_folders = get_subfolders(new_folders3);
    if invalid_folders.len() > 0 {
        let invalid = invalid_folders[0].to_owned();
        return Err(HttpError {
            result: f!(
                "Unable to select a folder that is a child of another selected folder: {invalid}"
            ),
        });
    }

    let connection = establish_connection().unwrap();
    //let sql = diesel::debug_query::<diesel::sqlite::Sqlite, _>(&folder.filter(get_path().ne_all(new_folders_req.folders.iter()).and(get_path().to_owned().ne("")))).to_string();
    //println!("{:?}", sql);
    let pred = folder.filter(
        get_path()
            .ne_all(new_folders_req.folders.iter())
            .and(get_path().ne("")),
    );
    let to_remove = pred
        .to_owned()
        .select(get_path())
        .load::<String>(&connection)
        .unwrap();
    let all_mounts = mount
        .select(get_mount_path())
        .load::<String>(&connection)
        .unwrap();
    for r in to_remove {
        let remove = all_mounts
            .iter()
            .filter(|m| r.starts_with(m.to_owned()))
            .collect::<Vec<_>>();
        let _ = diesel::delete(mount.filter(get_mount_path().eq_any(remove))).execute(&connection);
    }
    let res = diesel::delete(pred.to_owned()).execute(&connection);
    if res.is_err() {
        return Err(HttpError {
            result: "fail".to_owned(),
        });
    }
    let existing = folder
        .filter(get_path().eq_any(new_folders_req.folders.iter()))
        .load::<Folder>(&connection)
        .expect("error");

    let existing_paths = existing
        .iter()
        .map(|rr| get_platform_folder(rr).clone())
        .collect::<Vec<_>>();
    let folders_to_create = new_folders_req
        .folders
        .iter()
        .filter(|f| !existing_paths.contains(f))
        .map(|f| new_folder(f.to_owned()))
        .collect::<Vec<_>>();
    let res1 = diesel::insert_into(folder)
        .values(folders_to_create)
        .execute(&connection);
    if res1.is_err() {
        return Err(HttpError {
            result: "fail".to_owned(),
        });
    }
    let r = mount
        .select((unix_path, windows_path))
        .load::<(String, String)>(&connection)
        .unwrap()
        .iter()
        .map(|f| NtfsMapping {
            dir: f.0.to_owned(),
            drive: f.1.to_owned(),
        })
        .collect();
    sync_folder_mappings(r);
    return Ok(Json(()));
}

fn get_env_path() -> PathBuf {
    let path = dirs::config_dir().unwrap().join("platune");
    if !path.exists() {
        std::fs::create_dir_all(path.to_owned()).unwrap();
    }
    let full_path = path.join(".env");
    if !full_path.to_owned().exists() {
        std::fs::File::create(full_path.to_owned()).unwrap();
    }

    full_path
}

fn write_env(dir: &String) -> String {
    let mut file = File::create(get_env_path()).unwrap();
    let delim_escaped = get_delim_escaped();
    let escaped = dir.replace(get_delim(), get_delim_escaped());
    let full_url = f!("{escaped}{delim_escaped}platune.db");
    let _ = file.write_all(f!("DATABASE_URL={full_url}").as_bytes());
    return full_url;
}

#[api_v2_operation]
async fn update_db_path(request_json: Json<DirRequest>) -> Result<Json<()>, ()> {
    let request = request_json.into_inner();
    let full_url = write_env(&request.dir);
    let current_url_res = env::var(DATABASE_URL);
    if let Ok(current_url) = current_url_res {
        let _res = copy(current_url.to_owned(), full_url.to_owned());
        let _res2 = remove_file(current_url.to_owned());
    }
    env::set_var(DATABASE_URL, full_url);
    return Ok(Json(()));
}

fn sync_folder_mappings(mapping: Vec<NtfsMapping>) {
    let connection = establish_connection().unwrap();
    for r in mapping {
        if !IS_WINDOWS {
            let paths = folder
                .filter(full_path_unix.like(r.dir.to_owned() + "%"))
                .select((folder_id, full_path_unix))
                .load::<(i32, String)>(&connection)
                .unwrap();
            for path in paths {
                let mut replace_val = "".to_owned();
                if r.drive != "" {
                    let suffix = convert_delim_windows(path.1.replace(&r.dir, ""));
                    replace_val = f!("{r.drive}{suffix}");
                }
                let _ = diesel::update(folder.filter(folder_id.eq(path.0)))
                    .set(full_path_windows.eq(replace_val))
                    .execute(&connection);
            }
        } else {
            if r.drive == "" {
                continue;
            }
            let paths2 = folder
                .filter(full_path_windows.like(r.drive.to_owned() + "%"))
                .select((folder_id, full_path_windows))
                .load::<(i32, String)>(&connection)
                .unwrap();
            for path in paths2 {
                let suffix = convert_delim_unix(path.1.replace(&r.drive, ""));
                let replace_val = f!("{r.dir}{suffix}");
                let _ = diesel::update(folder.filter(folder_id.eq(path.0)))
                    .set(full_path_unix.eq(replace_val))
                    .execute(&connection);
            }

            let _ = diesel::update(
                folder.filter(
                    full_path_unix
                        .like(r.dir.to_owned() + "%")
                        .and(full_path_windows.not_like(r.drive.to_owned() + "%")),
                ),
            )
            .set(full_path_unix.eq(""))
            .execute(&connection);
        }
    }
}

#[api_v2_operation]
async fn get_songs(request_query: Query<SongRequest>) -> Result<Json<Vec<Song>>, ()> {
    let request = request_query.into_inner();
    let connection = establish_connection().unwrap();
    let mut query: diesel::query_builder::BoxedSelectStatement<_, _, diesel::sqlite::Sqlite> = song
        .into_boxed()
        .inner_join(artist)
        .inner_join(album)
        .inner_join(
            album_artist
                .on(schema::album_artist::album_artist_id.eq(schema::album::album_artist_id)),
        )
        .select((
            song_id,
            song_title,
            artist_name,
            album_artist_name,
            album_name,
            track_number,
            disc_number,
            duration,
            get_song_path(),
            diesel::dsl::sql::<Bool>("case when album_art is null then 0 else 1 end"),
        ));
    if let Some(limit) = request.limit {
        query = query.limit(limit);
    }
    if let Some(offset) = request.offset {
        query = query.offset(offset);
    }
    if let Some(req_artist_id) = request.artist_id {
        query = query.filter(crate::schema::artist::artist_id.eq(req_artist_id));
    }
    if let Some(req_album_artist_id) = request.album_artist_id {
        query = query.filter(crate::schema::album_artist::album_artist_id.eq(req_album_artist_id));
    }
    if let Some(req_album_id) = request.album_id {
        query = query.filter(crate::schema::album::album_id.eq(req_album_id));
    }
    if let Some(req_song_name) = &request.song_name {
        query = query.filter(crate::schema::song::song_title.eq(req_song_name));
    }
    let mut songs = query
        .load::<(
            i32,
            String,
            String,
            String,
            String,
            i32,
            i32,
            i32,
            String,
            bool,
        )>(&connection)
        .unwrap()
        .iter()
        .map(|s| Song {
            id: s.0,
            name: s.1.to_owned(),
            artist: s.2.to_owned(),
            album_artist: s.3.to_owned(),
            album: s.4.to_owned(),
            track: s.5,
            disc: s.6,
            time: s.7,
            path: s.8.to_owned(),
            has_art: s.9,
        })
        .collect::<Vec<_>>();
    &songs.sort_case_insensitive("album artist".to_owned());
    return Ok(Json(songs));
}

#[api_v2_operation]
async fn search(request_query: Query<Search>) -> Result<Json<Vec<SearchRes>>, ()> {
    let request = request_query.into_inner();
    let connection = establish_connection().unwrap();
    // If both album_artist and artist are returned by search, use ROW_NUMBER() to only return the artist
    // Adjust rankings to give slightly more weight to artists and albums

    // test cases: "red hot chili peppers" for album artist without artist
    // "fired up" for multiple artists with same album name
    let order_clause = "rank * (CASE entry_type WHEN 'artist' THEN 1.4 WHEN 'album_artist' THEN 1.4 WHEN 'album' THEN 1.25 ELSE 1 END)";
    let artist_select = "CASE entry_type WHEN 'song' THEN ar.artist_name WHEN 'album' THEN aa.album_artist_name ELSE NULL END";
    let res = diesel::sql_query(f!("
        WITH CTE AS (
            SELECT DISTINCT entry_value, entry_type, rank, 
            CASE entry_type WHEN 'song' THEN ar.artist_id WHEN 'album' THEN al.album_id ELSE assoc_id END correlation_id,
            {artist_select} artist,
            ROW_NUMBER() OVER (PARTITION BY entry_value, {artist_select}, CASE entry_type WHEN 'song' THEN 1 WHEN 'album' THEN 2 ELSE 3 END ORDER BY entry_type DESC) row_num
            FROM search_index
            LEFT OUTER JOIN song s on s.song_id = assoc_id
            LEFT OUTER JOIN artist ar on ar.artist_id = s.artist_id
            LEFT OUTER JOIN album al on al.album_id = assoc_id
            LEFT OUTER JOIN album_artist aa on aa.album_artist_id = al.album_artist_id
            WHERE search_index MATCH ?
            ORDER BY {order_clause}
            LIMIT ?
        )
        SELECT entry_value, entry_type, artist, correlation_id FROM cte
        WHERE row_num = 1
        ORDER BY {order_clause}
        LIMIT ?")
        )
        .bind::<diesel::sql_types::Text, _>(f!("entry_value:{request.search_string}"))
        .bind::<diesel::sql_types::Integer, _>(request.limit * 2)
        .bind::<diesel::sql_types::Integer, _>(request.limit)
        .load::<SearchRes>(&connection).unwrap();
    return Ok(Json(res));
}

#[api_v2_operation]
async fn get_album_art(request: Query<ArtRequest>) -> actix_http::Response {
    let connection = establish_connection().unwrap();
    let req_obj = request.into_inner();
    let art = song
        .filter(song_id.eq(req_obj.song_id))
        .select(album_art)
        .first::<Option<Vec<u8>>>(&connection)
        .unwrap();
    let mut builder = actix_http::Response::Ok();
    if art.is_some() {
        let resize = req_obj.width.is_some() && req_obj.height.is_some();
        let img = art.unwrap();
        let o = image::io::Reader::new(std::io::Cursor::new(&img))
            .with_guessed_format()
            .unwrap();
        let format = o.format().unwrap();
        if format == image::ImageFormat::Png {
            builder.set(actix_http::http::header::ContentType::png());
        } else {
            builder.set(actix_http::http::header::ContentType::jpeg());
        }
        if resize {
            let res = o.decode().unwrap().resize(
                req_obj.width.unwrap(),
                req_obj.height.unwrap(),
                image::imageops::FilterType::CatmullRom,
            );
            let mut buf = Vec::new();
            let write_res = res.write_to(&mut buf, format);
            return builder.body(buf);
        }
        return builder.body(img);
    } else {
        return builder.finish();
    }
}

fn get_brightness(rgb: &Rgb) -> f32 {
    return 0.2126 * (rgb.r as f32) + 0.7152 * (rgb.g as f32) + 0.0722 * (rgb.b as f32);
}

fn lighten(color: Rgb, correction_factor: f32) -> Rgb {
    //const correctionFactor = 0.5;
    let red = (255 - color.r) as f32 * correction_factor + color.r as f32;
    let green = (255 - color.g) as f32 * correction_factor + color.g as f32;
    let blue = (255 - color.b) as f32 * correction_factor + color.b as f32;
    let lighter_color = Rgb {
        r: red as u8,
        g: green as u8,
        b: blue as u8,
    };
    return lighter_color;
}

fn darken(color: Rgb, correction_factor: f32) -> Rgb {
    //const correctionFactor = 0.5;
    let red = color.r as f32 * (1.0 - correction_factor);
    let green = color.g as f32 * (1.0 - correction_factor);
    let blue = color.b as f32 * (1.0 - correction_factor);
    let darker_color = Rgb {
        r: red as u8,
        g: green as u8,
        b: blue as u8,
    };
    return darker_color;
}

fn adjust_darken(mut color: Rgb, luminance: f32, threshold: f32) -> Rgb {
    if luminance > threshold {
        let to_darken = 1.0 - threshold / luminance;
        color = darken(color, to_darken);
    }
    return color;
}

fn adjust_lighten(mut color: Rgb, luminance: f32, threshold: f32) -> Rgb {
    if luminance < threshold {
        let to_lighten = 1.0 - luminance / threshold;
        color = lighten(color, to_lighten);
    }
    return color;
}

#[api_v2_operation]
async fn get_art_colors(request_query: Query<ArtColorsRequest>) -> Result<Json<Vec<Rgb>>, ()> {
    let request = request_query.into_inner();
    let connection = establish_connection().unwrap();
    let art = song
        .filter(song_id.eq(request.song_id))
        .select(album_art)
        .first::<Option<Vec<u8>>>(&connection)
        .unwrap()
        .unwrap();

    let o = image::io::Reader::new(std::io::Cursor::new(&art))
        .with_guessed_format()
        .unwrap();
    let colors = 4 as usize;
    let num_colors = 3 as u8;
    let decoded = o.decode().unwrap();
    let rgba_img = decoded.to_rgba();
    let palette =
        color_thief::get_palette(&rgba_img, color_thief::ColorFormat::Rgba, 10, num_colors)
            .unwrap();
    let pal = palette
        .iter()
        .map(|p| Rgb {
            r: p.r,
            g: p.g,
            b: p.b,
        })
        .collect::<Vec<_>>();
    let mut temp = pal
        .iter()
        .clone()
        .map(get_brightness)
        .enumerate()
        .collect::<Vec<_>>();

    if request.is_light {
        let pal = palette
            .iter()
            .map(|p| Rgb {
                r: p.r,
                g: p.g,
                b: p.b,
            })
            .collect::<Vec<_>>();
        let mut temp = pal
            .iter()
            .clone()
            .map(get_brightness)
            .enumerate()
            .collect::<Vec<_>>();

        &mut temp.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let temp2 = temp.iter().rev().collect::<Vec<_>>();
        let bg_thresh = 180.0;
        let fg_thresh = 70.0;
        let secondary_min = 160.0;
        let secondary_max = 160.0;
        let bg = adjust_lighten(pal[temp2[0].0], temp2[0].1, bg_thresh);
        let fg = adjust_darken(pal[temp2[colors - 1].0], temp2[colors - 1].1, fg_thresh);
        let secondary = adjust_lighten(pal[temp2[1].0], temp2[1].1, secondary_min);
        let third = adjust_lighten(pal[temp2[2].0], temp2[2].1, secondary_min);
        let fourth = lighten(fg, 0.4);
        return Ok(Json(vec![bg, fg, secondary, third, fourth]));
    } else {
        &mut temp.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let bg_thresh = 70.0;
        let fg_thresh = 140.0;
        let secondary_min = 100.0;
        let secondary_max = 100.0;
        let bg = adjust_darken(pal[temp[0].0], temp[0].1, bg_thresh);
        let fg = adjust_lighten(pal[temp[colors - 1].0], temp[colors - 1].1, fg_thresh);
        let secondary = adjust_darken(pal[temp[1].0], temp[1].1, secondary_max);
        let third = adjust_darken(pal[temp[2].0], temp[2].1, secondary_max);
        let fourth = lighten(fg, 0.8);
        return Ok(Json(vec![bg, fg, secondary, third, fourth]));
    }
}

#[api_v2_operation]
async fn update_path_mappings(request: Json<Vec<NtfsMapping>>) -> Result<Json<()>, HttpError> {
    let connection = establish_connection().unwrap();
    let ins = request
        .iter()
        .map(|r| {
            (
                unix_path.eq(r.dir.to_owned()),
                windows_path.eq(r.drive.to_owned()),
            )
        })
        .collect::<Vec<_>>();
    let res = diesel::replace_into(mount).values(ins).execute(&connection);
    if res.is_err() {
        return Err(HttpError {
            result: "fail".to_owned(),
        });
    }
    let pred = mount.filter(unix_path.ne_all(request.iter().map(|r| r.dir.to_owned())));
    if IS_WINDOWS {
        let to_delete = pred
            .to_owned()
            .select(unix_path)
            .load::<String>(&connection)
            .unwrap();
        for d in to_delete {
            let _ = diesel::update(folder.filter(full_path_unix.like(d + "%")))
                .set(full_path_unix.eq(""))
                .execute(&connection);
        }
    }
    let res2 = diesel::delete(pred).execute(&connection);
    if res2.is_err() {
        return Err(HttpError {
            result: "fail".to_owned(),
        });
    }

    sync_folder_mappings(request.into_inner());
    return Ok(Json(()));
}

#[api_v2_operation]
async fn get_db_path() -> Result<Json<Dir>, ()> {
    let env_path = get_env_path();
    let mut file = File::open(env_path).unwrap();
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    let delim_escaped = get_delim_escaped();
    let delim = get_delim();
    let res = contents
        .split("=")
        .last()
        .unwrap()
        .replace(delim_escaped, delim)
        .replace(&f!("{delim}platune.db"), "");
    return Ok(Json(Dir {
        is_file: true,
        name: res.to_owned(),
    }));
}

#[api_v2_operation]
async fn add_tag(request_json: Json<TagRequest>) -> Result<Json<()>, ()> {
    let request = request_json.into_inner();
    let connection = establish_connection().unwrap();
    diesel::insert_into(tag)
        .values((
            tag_name.eq(request.name.to_owned()),
            tag_color.eq(request.color.to_owned()),
            tag_priority.eq(request.priority),
        ))
        .execute(&connection)
        .unwrap();
    return Ok(Json(()));
}

#[api_v2_operation]
async fn update_tag(
    request_path: Path<(i32,)>,
    request_json: Json<TagRequest>,
) -> Result<Json<()>, ()> {
    let request = request_json.into_inner();
    let id_path = request_path.into_inner();
    let connection = establish_connection().unwrap();
    diesel::update(tag.filter(tag_id.eq(id_path.0)))
        .set((
            tag_name.eq(request.name.to_owned()),
            tag_color.eq(request.color.to_owned()),
            tag_priority.eq(request.priority),
        ))
        .execute(&connection)
        .unwrap();
    return Ok(Json(()));
}

#[api_v2_operation]
async fn get_tags() -> Result<Json<Vec<TagResponse>>, ()> {
    let connection = establish_connection().unwrap();
    let tags = tag
        .select((tag_id, tag_name, tag_color, tag_priority))
        .load::<(i32, String, String, i32)>(&connection)
        .unwrap()
        .iter()
        .map(|t| TagResponse {
            id: t.0,
            name: t.1.to_owned(),
            color: t.2.to_owned(),
            priority: t.3,
        })
        .collect::<Vec<_>>();
    return Ok(Json(tags));
}

#[api_v2_operation]
async fn delete_tag(request_path: Path<(i32,)>) -> Result<Json<()>, ()> {
    let tag_id_req = request_path.into_inner();
    let connection = establish_connection().unwrap();
    diesel::delete(tag.filter(tag_id.eq(tag_id_req.0)))
        .execute(&connection)
        .unwrap();
    return Ok(Json(()));
}

#[derive(QueryableByName, Serialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct SearchRes {
    #[sql_type = "Text"]
    entry_value: String,
    #[sql_type = "Text"]
    entry_type: String,
    #[sql_type = "Nullable<Text>"]
    artist: Option<String>,
    #[sql_type = "Integer"]
    correlation_id: i32,
}

#[derive(Serialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct Dir {
    is_file: bool,
    name: String,
}

#[derive(Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct TagRequest {
    name: String,
    color: String,
    priority: i32,
}

#[derive(Serialize, Apiv2Schema, Queryable)]
#[serde(rename_all = "camelCase")]
struct TagResponse {
    id: i32,
    name: String,
    color: String,
    priority: i32,
}

#[derive(Serialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct DirResponse {
    dirs: Vec<Dir>,
}

#[derive(Serialize, Apiv2Schema, Copy, Clone)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct NtfsMapping {
    dir: String,
    drive: String,
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct Search {
    search_string: String,
    limit: i32,
}

#[derive(Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct DirRequest {
    dir: String,
}

#[derive(Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct FolderUpdate {
    folders: Vec<String>,
}

#[derive(Serialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: i32,
    pub path: String,
    pub artist: String,
    pub album_artist: String,
    pub name: String,
    pub album: String,
    pub track: i32,
    pub disc: i32,
    pub time: i32,
    pub has_art: bool,
}

#[derive(Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct SongRequest {
    offset: Option<i64>,
    limit: Option<i64>,
    artist_id: Option<i32>,
    album_artist_id: Option<i32>,
    album_id: Option<i32>,
    song_name: Option<String>,
}

#[derive(Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct ArtRequest {
    song_id: i32,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct ArtColorsRequest {
    song_id: i32,
    is_light: bool,
}
