#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket; // using rocket 0.4.10

use std::sync::Mutex;

use rocket_contrib::json::Json;
use serde::Deserialize;

use rocket::State;
use rocket::http::Status;
use rocket::http::Method;

use rocket_cors::{AllowedOrigins, CorsOptions};

use rocket_contrib::templates::Template;
use std::collections::HashMap;

#[derive(Deserialize)]
struct ClipboardContents<'r> {
    text: &'r str
}

#[get("/")]
fn index(clipboard: State<Mutex<String>>) -> Template {
    let data = clipboard.lock().unwrap();
    let context: HashMap<&str, &str> = [("clipboard", data.as_str())].iter().cloned().collect();
    Template::render("index", &context)
}

#[get("/get_clipboard")]
fn get_clipboard(clipboard: State<Mutex<String>>) -> String {
    format!("{}", clipboard.lock().unwrap().as_str())
}

#[post("/set_clipboard", format = "application/json", data="<contents>")]
fn set_clipboard(contents: Json<ClipboardContents<'_>>, clipboard: State<Mutex<String>>) -> Status {    
    let mut data = clipboard.lock().unwrap();
    data.clear();
    data.push_str(contents.text);
    Status::Accepted
}

fn main() {

    let cors = CorsOptions::default()
    .allowed_origins(AllowedOrigins::all())
    .allowed_methods(
        vec![Method::Get, Method::Post, Method::Patch]
            .into_iter()
            .map(From::from)
            .collect(),
    ).allow_credentials(true);
    
    rocket::ignite()
	.mount("/", routes![index])
	.mount("/api", routes![set_clipboard, get_clipboard])
	.manage(Mutex::new(String::from("Init string")))
	.attach(cors.to_cors().unwrap())
	.attach(Template::fairing()).launch();
}
