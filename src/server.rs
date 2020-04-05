use dirs::home_dir;
use std::{sync::mpsc};
use actix_web::{dev::Server, HttpServer, HttpRequest, HttpResponse, App, http::Method, web, Result, http::StatusCode, get, error};
use actix_cors::Cors;
use actix_files as fs;
use serde::{Deserialize, Serialize};

pub fn run_server(tx: mpsc::Sender<Server>) -> std::io::Result<()> {
    let mut sys = actix_rt::System::new("test");

    let srv = HttpServer::new(|| { 
        App::new()
        .wrap(
            Cors::new()
                .allowed_origin("http://localhost:3000")
                .finish()
        )
        .service(fs::Files::new("/music", "//home/aschey/windows/shared_files/Music").show_files_listing())
        .service(get_home_dir)
        .service(get_configured_folders)
        .service(fs::Files::new("/", "./src/ui/namp/build").show_files_listing())
        })
        .bind("127.0.0.1:5000")?
        .run();

    // send server controller to main thread
    let _ = tx.send(srv.clone());

    // run future
    sys.block_on(srv)
}

#[get("/homeDir")]
async fn get_home_dir(_: HttpRequest) -> Option<Result<HttpResponse>> {
    let home_path = home_dir()?;
    let home_str = String::from(home_path.to_str()?);

    let response = HttpResponse::build(StatusCode::OK)
        //.content_type("application/json; charset=utf-8")
        .json(HomeDirResponse {home_dir: home_str});
    return Some(Ok(response));
}

#[get("/configuredFolders")]
async fn get_configured_folders() -> Result<HttpResponse> {
    return Ok(HttpResponse::build(StatusCode::OK).json(["/home/aschey", "/shared/stuff"]));
}

#[derive(Serialize)]
struct HomeDirResponse {
    home_dir: String,
}