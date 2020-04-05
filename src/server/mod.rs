use dirs::home_dir;
use std::{sync::mpsc};
use actix_web::{dev::Server, HttpServer, HttpRequest, HttpResponse, App, http::Method, Result, http::StatusCode, get, error};
use actix_cors::Cors;
use actix_files as fs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::vec::Vec;
use paperclip::actix::{
    // extension trait for actix_web::App and proc-macro attributes
    OpenApiExt, Apiv2Schema, api_v2_operation,
    // use this instead of actix_web::web
    web::{self, Json},
};

pub fn run_server(tx: mpsc::Sender<Server>) -> std::io::Result<()> {
    let mut sys = actix_rt::System::new("test");

    let srv = HttpServer::new(|| { 
        App::new()
        .wrap(Cors::new().allowed_origin("http://localhost:3000").finish())
        .wrap_api()
        // REST endpoints
        .service(web::resource("/homeDir").route(web::get().to(get_home_dir)))
        .service(web::resource("/configuredFolders").route(web::get().to(get_configured_folders)))
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

#[api_v2_operation]
async fn get_home_dir(_: HttpRequest) -> Option<Result<Json<HomeDirResponse>, ()>> {
    let home_path = home_dir()?;
    let home_str = String::from(home_path.to_str()?);

    //let response = HttpResponse::build(StatusCode::OK)
        let response = Json(HomeDirResponse {home_dir: home_str});
    return Some(Ok(response));
}

#[api_v2_operation]
async fn get_configured_folders() -> Result<Json<Vec<String>>, ()> {
    let a = String::from("/home/aschey");
    let xs = vec![a];
    return Ok(Json(xs));
}

#[derive(Serialize, Apiv2Schema)]
struct HomeDirResponse {
    home_dir: String,
}