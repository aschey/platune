use dirs::home_dir;
use std::{sync::mpsc};
use actix_web::{dev::Server, HttpServer, HttpRequest, HttpResponse, App, http::Method, web, Result, http::StatusCode, get, error};
use actix_files as fs;

pub fn run_server(tx: mpsc::Sender<Server>) -> std::io::Result<()> {
    let mut sys = actix_rt::System::new("test");

    // srv is server controller type, `dev::Server`
    let srv = HttpServer::new(|| { 
        App::new()
        .service(fs::Files::new("/music", "//home/aschey/windows/shared_files/Music").show_files_listing())
        .service(get_home_dir)
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
        .content_type("text/html; charset=utf-8")
        .body(home_str);
    return Some(Ok(response));
}