#[macro_use] extern crate rocket; 

use std::sync::Mutex;
use rocket::State;

use rocket::http::Status;
use rocket::serde::{Deserialize, json::Json, Serialize};
use rocket_dyn_templates::Template;

use rocket::fs::FileServer;

use std::collections::HashMap;

#[derive(Deserialize)]
struct ClipboardContents {
    text: String    
}

#[derive(Serialize)]
struct TemplateContext {
    clipboards: HashMap<String,String>
}

#[post("/kill_clipboard/<hostname>")]
fn kill_clipboard(clipboard: &State<Mutex<HashMap<String,String>>>, hostname: String) -> Status {
    let mut data = clipboard.lock().unwrap();
    data.remove(&hostname);
    Status::Accepted
}

#[get("/get_clipboard/<hostname>")]
fn get_clipboard(clipboard: &State<Mutex<HashMap<String,String>>>, hostname: String) -> String {   
    let data = clipboard.lock().unwrap();    
    format!("{}", data[&hostname].as_str())
}

#[get("/get_clipboards")]
fn get_clipboards(clipboard: &State<Mutex<HashMap<String,String>>>) -> Json<HashMap<String, String>> {
    let data = clipboard.lock().unwrap();    
    Json(data.clone())
}

#[post("/set_clipboard/<hostname>", format = "application/json", data="<contents>")]
fn set_clipboard(contents: Json<ClipboardContents>, clipboard: &State<Mutex<HashMap<String,String>>>, hostname: String) -> Status {    
    let mut data = clipboard.lock().unwrap();
    data.insert(hostname, contents.text.to_string());    
    Status::Accepted
}

#[get("/")]
pub fn index(clipboard: &State<Mutex<HashMap<String,String>>>) -> Template {
    let data = clipboard.lock().unwrap();
            
    Template::render("index", &TemplateContext {
	clipboards: data.clone()
    })
}

#[launch]
fn rocket() -> _ {

    let mut hmap = HashMap::<String,String>::new();
    hmap.insert("Server Clipboard".to_string(), "Initial String".to_string());
    
    rocket::build()
	.mount("/", routes![index])
	.mount("/api", routes![set_clipboard, get_clipboards, get_clipboard, kill_clipboard])
	.mount("/res", FileServer::from("static"))
	.manage(Mutex::new(hmap))        
	.attach(Template::fairing())
}
