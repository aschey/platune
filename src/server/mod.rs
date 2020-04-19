mod models;
use dirs::home_dir;
use std::{sync::mpsc, str};
use actix_web::{dev::Server, HttpServer, HttpRequest, HttpResponse, App, http::{Method, header}, Result, http::StatusCode, get, error, web::Query, Responder, body};
use actix_cors::Cors;
use actix_files as fs;
use serde::{Deserialize, Serialize};
use std::fs::{read_dir, DirEntry, File, copy, remove_file};
use std::path::PathBuf;
use dotenv::dotenv;
use std::{vec::Vec, env};
use diesel;
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use models::folder::*;
use crate::schema::folder::dsl::*;
use crate::schema;
use sysinfo::{ProcessExt, SystemExt, DiskExt};
use itertools::Itertools;
use fstrings::*;
use std::io::prelude::*;

use paperclip::actix::{
    // extension trait for actix_web::App and proc-macro attributes
    OpenApiExt, Apiv2Schema, api_v2_operation,
    // use this instead of actix_web::web
    web::{self, Json},
    api_v2_errors
};
use failure::Fail;

const IS_WINDOWS: bool = cfg!(windows);
const DATABASE_URL: &str = "DATABASE_URL";

#[cfg(windows)]
fn get_path() -> schema::folder::full_path_windows { full_path_windows }
#[cfg(unix)]
fn get_path() -> schema::folder::full_path_unix { full_path_unix }

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = env::var(DATABASE_URL)
        .expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect("Error connecting to database")

}

pub fn run_server(tx: mpsc::Sender<Server>) -> std::io::Result<()> {
    let mut sys = actix_rt::System::new("test");

    let srv = HttpServer::new(|| { 
        App::new()
        .wrap(Cors::new().finish())
        .wrap_api()
        // REST endpoints
        .service(web::resource("/dirsInit").route(web::get().to(get_dirs_init)))
        .service(web::resource("/dirs").route(web::get().to(get_dirs)))
        .service(web::resource("/configuredFolders").route(web::get().to(get_configured_folders)))
        .service(web::resource("/isWindows").route(web::get().to(get_is_windows)))
        .service(web::resource("/updateFolders").route(web::put().to(update_folders)))
        .service(web::resource("/getDbPath").route(web::get().to(get_db_path)))
        .service(web::resource("/updateDbPath").route(web::put().to(update_db_path)))
        .service(web::resource("/getNtfsMounts").route(web::get().to(get_ntfs_mounts)))
        .with_json_spec_at("/spec")
        .build()
        // static files
        .service(fs::Files::new("/swagger", "./src/ui/namp/swagger").index_file("index.html"))
        .service(fs::Files::new("/music", "//home/aschey/windows/shared_files/Music").show_files_listing())
        // Paths are matched in order so this needs to be last
        .service(fs::Files::new("/", "./src/ui/namp/build").show_files_listing())
        })
        .bind("127.0.0.1:5000")?
        .run();

    // send server controller to main thread
    let _ = tx.send(srv.clone());

    // run future
    sys.block_on(srv)
}

fn filter_dirs(res: Result<DirEntry, std::io::Error>, delim: &str) -> Option<Dir> {
    let path = res.unwrap().path();
    let str_path = String::from(path.to_str().unwrap());
    let dir_name = String::from(str_path.split(delim).last().unwrap());
    if !dir_name.starts_with(".") { Some(Dir {name: dir_name, is_file: path.is_file() }) } else { None }
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

#[api_v2_errors(
    code=400,
    code=401, description="Unauthorized: Can't read session from header",
    code=500,
)]
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
    // let d = system.get_disks();
    // for disk in system.get_disks() {
    //          println!("{:?}", std::str::from_utf8(disk.get_file_system()).unwrap() == "fuseblk");
    //     }
    let disks = system.get_disks().iter().map(|d| Dir { is_file: false, name: get_dir_name(d.get_mount_point()) }).collect::<Vec<_>>();
    return Ok(Json(DirResponse {dirs: disks}))
}

#[api_v2_operation]
async fn get_dirs(dir_request: Query<DirRequest>) -> Result<Json<DirResponse>, ()> {
    let delim = if IS_WINDOWS { "\\" } else { "/" };
    let mut entries = read_dir(dir_request.dir.as_str()).unwrap()
        .filter_map(|res| filter_dirs(res, delim))
        .collect::<Vec<_>>();

    entries.sort_case_insensitive();
    let response = Json(DirResponse {dirs: entries});
    return Ok(response);
}

#[api_v2_operation]
async fn get_is_windows() -> Result<Json<bool>, ()> {
    return Ok(Json(IS_WINDOWS));
}

fn get_configured_folders_helper() -> Vec<String> {
    let connection = establish_connection();
    let results = folder.load::<Folder>(&connection).expect("error");
    let paths = results.iter().map(|rr| get_platform_folder(rr).clone()).collect();
    return paths;
}

#[api_v2_operation]
async fn get_configured_folders() -> Result<Json<Vec<String>>, ()> {
    let paths = get_configured_folders_helper();
    return Ok(Json(paths));
}

#[api_v2_operation]
async fn get_ntfs_mounts() -> Result<Json<Vec<String>>, ()> {
    let system = sysinfo::System::new_all();
    let disks = system.get_disks();
    let fuse_disks = disks.iter()
        .filter(|d| str::from_utf8(d.get_file_system()).unwrap() == "fuseblk")
        .map(|d| get_dir_name(d.get_mount_point()))
        .collect::<Vec<_>>();
    let configured = get_configured_folders_helper();
    let configured_fuse = fuse_disks.into_iter().filter(|f| configured.iter().any(|c| c.starts_with(f))).collect::<Vec<_>>();
    return Ok(Json(configured_fuse.to_owned()));
    // for disk in system.get_disks() {
    //          println!("{:?}", std::str::from_utf8(disk.get_file_system()).unwrap() == "fuseblk");
    //     }
}

fn get_subfolders(new_folders: Vec<String>) -> Vec<String> {
    let copy = new_folders.to_vec();
    let dedup = &new_folders.into_iter().dedup_by(|l, r| r.starts_with(l)).collect::<Vec<_>>();
    
    let lala = copy.into_iter().filter(|f| !dedup.contains(f)).collect::<Vec<_>>();
    return lala;
}

fn get_dupe_folders(new_folders: Vec<String>) -> Vec<(String, Vec<String>)> {
    let grouped = new_folders.into_iter().group_by(|f| String::from(f)).into_iter().map(|(key, group)| (key, group.collect::<Vec<_>>())).collect::<Vec<(String, Vec<String>)>>();
    return grouped;
}

fn get_platform_folder(f: &Folder) -> String {
    if IS_WINDOWS { f.full_path_windows.to_owned() } else { f.full_path_unix.to_owned() }
}

fn new_folder(path: String) -> NewFolder {
    if IS_WINDOWS {
        NewFolder {
            full_path_unix: "".to_owned(),
            full_path_windows: path
        }
    }
    else {
        NewFolder {
            full_path_unix: path,
            full_path_windows: "".to_owned()
        }
    }
}

#[api_v2_operation]
async fn update_folders(new_folders_req: Json<FolderUpdate>) -> Result<Json<()>, HttpError> {
    let mut new_folders = new_folders_req.folders.to_vec();
    new_folders.sort_case_insensitive();
    
    let new_folders2 = new_folders.to_vec();
    let new_folders3 = new_folders.to_vec();
    let grouped = get_dupe_folders(new_folders);
    for (_, group) in grouped.into_iter() {
        if group.len() > 1 {
            let dup = group[0].to_owned();
            return Err(HttpError {result: f!("Duplicate folder chosen: {dup}")});
        }
    }

    let invalid_folders = get_subfolders(new_folders3);
    if invalid_folders.len() > 0 {
        let invalid = invalid_folders[0].to_owned();
        return Err(HttpError {result: f!("Unable to select a folder that is a child of another selected folder: {invalid}")});
    }

    let connection = establish_connection();
    let res = diesel::delete(folder.filter(get_path().ne_all(new_folders_req.folders.iter()))).execute(&connection);
    if res.is_err() {
        return Err(HttpError {result: "fail".to_owned()});
    }
    let existing = folder
        .filter(get_path().eq_any(new_folders_req.folders.iter()))
        .load::<Folder>(&connection).expect("error");
        
    let existing_paths = existing.iter().map(|rr| get_platform_folder(rr).clone()).collect::<Vec<_>>();
    let folders_to_create = new_folders_req.folders.iter()
        .filter(|f| !existing_paths.contains(f))
        .map(|f| new_folder(f.to_owned())).collect::<Vec<_>>();
    let res1 = diesel::insert_into(folder).values(folders_to_create).execute(&connection);
    if res1.is_err() {
        return Err(HttpError {result: "fail".to_owned()});
    }
    return Ok(Json(()));
}

#[api_v2_operation]
async fn update_db_path(request: Json<DirRequest>) -> Result<Json<()>, ()> {
    let mut file = File::create(".env").unwrap();
    let full_url = f!("{request.dir}/namp.db");
    let _ = file.write_all(f!("DATABASE_URL={full_url}").as_bytes());
    let current_url = env::var(DATABASE_URL).unwrap();
    let _res = copy(current_url.to_owned(), full_url.to_owned());
    let _res2 = remove_file(current_url.to_owned());
    env::set_var(DATABASE_URL, full_url);
    return Ok(Json(()));
}

#[api_v2_operation]
async fn get_db_path() -> Result<Json<Dir>, ()>{
    let mut file = File::open(".env").unwrap();
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    let res = contents.split("=").last().unwrap().replace("/namp.db", "");
    return Ok(Json(Dir { is_file: true, name: res.to_owned()}));
}

#[derive(Serialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct Dir {
    is_file: bool,
    name: String
}

#[derive(Serialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct DirResponse {
    dirs: Vec<Dir>,
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