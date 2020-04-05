use std::sync::mpsc;
use actix_web::{dev::Server, HttpServer, HttpRequest, HttpResponse, App, http::Method, web};
use actix_files as fs;

pub fn run_server(tx: mpsc::Sender<Server>) -> std::io::Result<()> {
    let mut sys = actix_rt::System::new("test");

    // srv is server controller type, `dev::Server`
    let srv = HttpServer::new(|| { 
        App::new()
        .service(fs::Files::new("/music", "//home/aschey/windows/shared_files/Music").show_files_listing())
        .service(
            web::resource("/test").to(|req: HttpRequest| match *req.method() {
                Method::GET => HttpResponse::Ok(),
                Method::POST => HttpResponse::MethodNotAllowed(),
                _ => HttpResponse::NotFound(),
            }),
        )
        .service(fs::Files::new("/", "./src/ui/namp/build").show_files_listing())
        })
        .bind("127.0.0.1:5000")?
        .run();

    // send server controller to main thread
    let _ = tx.send(srv.clone());

    // run future
    sys.block_on(srv)
}