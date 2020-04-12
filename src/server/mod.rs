mod models;
use dirs::home_dir;
use std::{sync::mpsc};
use actix_web::{dev::Server, HttpServer, HttpRequest, HttpResponse, App, http::{Method, header}, Result, http::StatusCode, get, error, web::Query, Responder, body};
use actix_cors::Cors;
use actix_files as fs;
use serde::{Deserialize, Serialize};
use std::fs::{read_dir, DirEntry};
use std::path::PathBuf;
use dotenv::dotenv;
use std::{vec::Vec, env};
use diesel;
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use models::folder::*;
use crate::schema::folder::dsl::*;
use sysinfo::{ProcessExt, SystemExt, DiskExt};
use itertools::Itertools;

use paperclip::actix::{
    // extension trait for actix_web::App and proc-macro attributes
    OpenApiExt, Apiv2Schema, api_v2_operation,
    // use this instead of actix_web::web
    web::{self, Json},
    api_v2_errors
};
use failure::Fail;

const IS_WINDOWS: bool = cfg!(windows);

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
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
        .with_json_spec_at("/docs")
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

fn filter_dirs(res: Result<DirEntry, std::io::Error>, delim: &str) -> Option<String> {
    let path = res.unwrap().path();
    if !path.is_dir() {
        return None
    }
    let str_path = String::from(path.to_str().unwrap());
    let dir_name = String::from(str_path.split(delim).last().unwrap());
    if !dir_name.starts_with(".") { Some(dir_name) } else { None }
}

pub trait StringVecExt {
    fn sort_case_insensitive(&mut self);
}

impl StringVecExt for Vec<String> {
    fn sort_case_insensitive(&mut self) {
        &self.sort_by(|l, r| Ord::cmp(&l.to_lowercase(), &r.to_lowercase()));
    }
}

#[api_v2_errors(
    code=400,
    code=401, description="Unauthorized: Can't read session from header",
    code=500,
)]
#[derive(Fail, Debug)]
#[fail(display = "my error")]
struct MyError {
    name: String,
}

// Use default implementation for `error_response()` method
impl error::ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        let mut resp = HttpResponse::new(self.status_code());
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        resp.set_body(body::Body::from(self.name.to_owned()))
    }
}


#[api_v2_operation]
async fn get_dirs_init() -> Result<Json<DirResponse>, ()> {
    let system = sysinfo::System::new_all();
    let mut disks = system.get_disks().iter().map(|d| String::from(d.get_mount_point().to_str().unwrap())).collect::<Vec<_>>();
    if IS_WINDOWS {
        disks = disks.iter().map(|d| d.replace("\\", "")).collect();
    }
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

#[api_v2_operation]
async fn get_configured_folders() -> Result<Json<Vec<String>>, ()> {
    let connection = establish_connection();
    let results = folder.load::<Folder>(&connection).expect("error");
    let paths = results.iter().map(|rr| rr.full_path.clone()).collect();
    
    return Ok(Json(paths));
}

fn get_subfolders(new_folders: Vec<String>) -> Vec<String> {
    let copy = new_folders.to_vec();
    // |l, r| r.starts_with(l)
    let dedup = &new_folders.into_iter().dedup_by(|l, r| r.starts_with(l)).collect::<Vec<_>>();
    //let lala = copy.iter().filter(|&f| !dedup.contains(f)).collect::<Vec<_>>();
    
    let lala = copy.into_iter().filter(|f| !dedup.contains(f)).collect::<Vec<_>>();
    return lala;
    // for _l in lala {
    //     return Err(());
    // }
}

fn get_dupe_folders(new_folders: Vec<String>) -> Vec<(String, Vec<String>)> {
    let grouped = new_folders.into_iter().group_by(|f| String::from(f)).into_iter().map(|(key, group)| (key, group.collect::<Vec<_>>())).collect::<Vec<(String, Vec<String>)>>();
    return grouped;
}

#[api_v2_operation]
async fn update_folders(new_folders_req: Json<FolderUpdate>) -> Result<Json<()>, MyError> {
    let mut new_folders = new_folders_req.folders.to_vec();
    new_folders.sort_case_insensitive();
    //let t = new_folders.to_vec();
    //let new_folders = &new_folders_req.folders;
    //let data = vec![String::from("a"), String::from("b")];
    //let mut data_grouped = Vec::new();
    // for (key, group) in &data.into_iter().group_by(|elt| *elt >= 0) {
    //     data_grouped.push((key, group.collect::<Vec<_>>()));
    // }
    let new_folders2 = new_folders.to_vec();
    let new_folders3 = new_folders.to_vec();
    //let grouped = test(new_folders, |l, r| l == r);
    let grouped = get_dupe_folders(new_folders);//&new_folders.into_iter().group_by(|f| String::from(f)).into_iter().map(|(key, group)| (key, group.collect::<Vec<_>>())).collect::<Vec<(String, Vec<String>)>>();
    for (_, group) in grouped.into_iter() {
        if group.len() > 1 {
            return Err(MyError {name: "fail".to_owned()});
        }
    }
    //let dedup = &new_folders2.into_iter().dedup_by(|l, r| r.starts_with(l)).collect::<Vec<_>>();
    //let lala = new_folders3.iter().filter(|f| !dedup.contains(*f)).collect::<Vec<_>>();
    let lala = get_subfolders(new_folders3);
    for _l in lala {
        return Err(MyError {name: "fail".to_owned()});
    }

    let connection = establish_connection();
    let res = diesel::delete(folder.filter(full_path.ne_all(new_folders_req.folders.iter()))).execute(&connection);
    if res.is_err() {
        return Err(MyError {name: "fail".to_owned()});
    }
    let existing = folder.filter(full_path.eq_any(new_folders_req.folders.iter())).load::<Folder>(&connection).expect("error");
    let existing_paths = existing.iter().map(|rr| rr.full_path.clone()).collect::<Vec<_>>();
    let folders_to_create = new_folders_req.folders.iter().filter(|f| !existing_paths.contains(f)).map(|f| full_path.eq(f)).collect::<Vec<_>>();
    let res1 = diesel::insert_into(folder).values(folders_to_create).execute(&connection);
    if res1.is_err() {
        return Err(MyError {name: "fail".to_owned()});
    }
    return Ok(Json(()));
}

#[derive(Serialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
struct DirResponse {
    dirs: Vec<String>,
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