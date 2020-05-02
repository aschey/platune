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

fn stop_server(srv: actix_server::Server, t: std::thread::JoinHandle<()>) {
    actix_rt::System::new("").block_on(srv.stop(true));
    let _ = t.join().unwrap();
}

fn main() {
    let direct = false;
    let (tx, rx) = mpsc::channel();
    
    if direct {
        let _ = server::run_server(tx);
    }
    else {
        let t = thread::spawn(move || {
            //task::block_on(server::get_all_files());
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
                stop_server(srv, t);
                println!("exiting...");
                std::process::exit(0);
            }
            if let Ok(_) = listener.accept() {
                stop_server(srv, t);
                println!("exiting...");
                std::process::exit(0);
            }
        }
    }
}
