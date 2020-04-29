//#![windows_subsystem = "windows"]

mod server;
pub mod schema;

#[macro_use]
extern crate diesel;

use std::{thread, time::Duration};
use subprocess::{Exec};
use std::sync::mpsc;
use fstrings::*;
use async_std::task;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
const IS_WINDOWS: bool = cfg!(windows);
const IS_DEBUG: bool = cfg!(debug_assertions);

fn main() {
    //for file in files.unwrap() {
    //     let secs = file.unwrap().path().metadata().unwrap().modified().unwrap();
    //     let dur = secs.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64;
    //     let dt = chrono::Utc.timestamp(dur, 0);
    //     println!("{:?}", dt.to_rfc2822());
    // }
    //task::block_on(server::get_all_files());
    //return;
    // let port = if IS_DEBUG { 3000 } else { 5000 };
    // let content_url = f!("http://localhost:{port}/index.html");

    // if IS_DEBUG {
    //     ensure_node_started();
    // }
    let direct = false;
    let (tx, rx) = mpsc::channel();
    if direct {
        let _ = server::run_server(tx);
    }
    else {
        thread::spawn(move || {
            let _ = server::run_server(tx);
        });
        let srv = rx.recv().unwrap();
        let listener = TcpListener::bind("127.0.0.1:8002").unwrap();
        listener.set_nonblocking(true).expect("Cannot set non-blocking");
        loop {
            println!("waiting...");
            std::thread::sleep(std::time::Duration::from_secs(5));
            if let Ok(_) = TcpStream::connect("127.0.0.1:8001") {
                println!("response from server");
            } else {
                actix_rt::System::new("").block_on(srv.stop(true));
                println!("exiting...");
                std::process::exit(0);
            }
            if let Ok(_) = listener.accept() {
                actix_rt::System::new("").block_on(srv.stop(true));
                println!("exiting...");
                std::process::exit(0);
            }
        }
    }
}
