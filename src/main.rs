//#![windows_subsystem = "windows"]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate web_view;
#[macro_use]
extern crate fstrings;

use web_view::*;
use actix_files as fs;
use actix_web::{App, HttpServer};
use std::{thread, time::Duration};
use subprocess::{Exec};


fn main() {
    let is_production = false;
    let port = if is_production { 5000 } else { 3000 };
    let content_url = f!("http://localhost:{port}/index.html");
    let res = Exec::shell("pgrep -f yarn").capture().unwrap().stdout_str();
    if res.chars().count() == 0 {
        thread::spawn(move || {
            let _ = Exec::shell("cd src/ui/namp && yarn run start").join();
        });
        thread::sleep(Duration::from_millis(100));
    }
    
    let handle = thread::spawn(move || {
        let sys = actix_rt::System::new("actix-example");

        let server = HttpServer::new(|| { 
            App::new()
            .service(fs::Files::new("/music", "//home/aschey/windows/shared_files/Music").show_files_listing())
            .service(fs::Files::new("/", "./src/ui/namp/build").show_files_listing())
            })
            .bind("127.0.0.1:5000")
            .unwrap();

        server.run();
        let _ = sys.run();
    });

    let mut webview = web_view::builder()
        .title("Rust Todo App")
        .content(Content::Url(content_url))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(vec![])
        .invoke_handler(|webview, arg| {
            use Cmd::*;

            let tasks_len = {
                let tasks = webview.user_data_mut();

                match serde_json::from_str(arg).unwrap() {
                    Init => (),
                    Log { text } => println!("{}", text),
                    AddTask { name } => tasks.push(Task { name, done: false }),
                    MarkTask { index, done } => tasks[index].done = done,
                    ClearDoneTasks => tasks.retain(|t| !t.done),
                }

                tasks.len()
            };

            webview.set_title(&format!("Rust Todo App ({} Tasks)", tasks_len))?;
            render(webview)
        })
        .build()
        .unwrap();

    webview.set_color((156, 39, 176));

    let res = webview.run().unwrap();
    let _ = handle.join();
    println!("final state: {:?}", res);
}

fn render(webview: &mut WebView<Vec<Task>>) -> WVResult {
    let render_tasks = {
        let tasks = webview.user_data();
        println!("{:#?}", tasks);
        format!("rpc.render({})", serde_json::to_string(tasks).unwrap())
    };
    webview.eval(&render_tasks)
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    name: String,
    done: bool,
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    Init,
    Log { text: String },
    AddTask { name: String },
    MarkTask { index: usize, done: bool },
    ClearDoneTasks,
}
