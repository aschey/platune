//#![windows_subsystem = "windows"]

mod server;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate fstrings;
extern crate serde_json;
extern crate web_view;
extern crate dirs;


use web_view::*;
use actix_files as fs;
use actix_web::{App, HttpServer, get, Responder, HttpRequest, HttpResponse, http::StatusCode, Result, web, http::Method, dev::Server};
use std::{thread, time::Duration};
use subprocess::{Exec};
use std::sync::mpsc;


fn main() {
    let is_production = false;
    let port = if is_production { 5000 } else { 3000 };
    let content_url = f!("http://localhost:{port}/index.html");
    let res = Exec::shell("pgrep -f yarn").capture().unwrap().stdout_str();
    if res.chars().count() == 0 {
        thread::spawn(move || {
            let _ = Exec::shell("cd src/ui/namp && yarn run start").join();
        });
        thread::sleep(Duration::from_millis(500));
    }
    
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = server::run_server(tx);
    });
    let srv = rx.recv().unwrap();

    let mut webview = web_view::builder()
        .title("Rust Todo App")
        .content(Content::Url(content_url))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .build()
        .unwrap();

    webview.set_color((156, 39, 176));

    let res = webview.run().unwrap();
    actix_rt::System::new("").block_on(srv.stop(true));
    println!("final state: {:?}", res);
}
