//#![windows_subsystem = "windows"]

mod server;

use web_view::*;
use std::{thread, time::Duration};
use subprocess::{Exec};
use std::sync::mpsc;
use fstrings::*;


fn main() {
    let is_production = false;
    let port = if is_production { 5000 } else { 3000 };
    let content_url = f!("http://localhost:{port}/index.html");

    if !is_production {
        ensure_node_started();
    }
    
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = server::run_server(tx);
    });
    let srv = rx.recv().unwrap();
    let webview = web_view::builder()
        .title("NAMP")
        .content(Content::Url(content_url))
        // There's no maximize function so just set it to something large
        .size(100000, 100000)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .build()
        .unwrap();

    let _ = webview.run();
    actix_rt::System::new("").block_on(srv.stop(true));
}

fn ensure_node_started() {
    // TODO: make this more robust
    let res = Exec::shell("pgrep -f yarn").capture().unwrap().stdout_str();
        if res.chars().count() == 0 {
            // This should only run once until the node server is manually stopped
            thread::spawn(move || {
                let _ = Exec::shell("cd src/ui/namp && yarn run start").join();
            });
            // Wait for the node dev server to start
            thread::sleep(Duration::from_millis(2000));
        }
}