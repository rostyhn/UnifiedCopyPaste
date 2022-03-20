#[macro_use]
extern crate rocket;

use rocket::fs::FileServer;
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use rocket_dyn_templates::Template;

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UnifiedClipboard {
    contents: String,
    passphrase: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Auth {
    passphrase: String,
}

#[derive(Serialize)]
struct TemplateContext {
    clipboards: HashMap<String, UnifiedClipboard>,
}

#[post("/kill_clipboard/<hostname>", format = "application/json", data = "<auth>")]
fn kill_clipboard(
    auth: Json<Auth>,
    clipboard: &State<Mutex<HashMap<String, UnifiedClipboard>>>,
    hostname: String,
) -> Status {

    let mut clipboards = clipboard.lock().unwrap();    
    let this_clip = clipboards.get_mut(&hostname).unwrap();
    
    if this_clip.passphrase.is_empty() || auth.passphrase == this_clip.passphrase {
        clipboards.remove(&hostname);
        Status::Ok
    } else {
        Status::Forbidden
    }
}

#[get("/get_clipboard/<hostname>", format = "application/json", data = "<auth>")]
fn get_clipboard(clipboard: &State<Mutex<HashMap<String, UnifiedClipboard>>>, hostname: String, auth: Json<Auth>) -> String {
    let data = clipboard.lock().unwrap();    
    
    if !data.contains_key(&hostname) {
        return "Clipboard does not exist.".to_string()
    }

    let this_clip = &data[&hostname];

    if this_clip.passphrase.is_empty() || auth.passphrase == this_clip.passphrase {    
        data[&hostname].contents.clone()
    } else {
        // should return Status or string
        return "Invalid passphrase.".to_string()
    }
}

#[get("/get_clipboards")]
fn get_clipboards(
    clipboard: &State<Mutex<HashMap<String, UnifiedClipboard>>>,
) -> Json<Vec<String>> {
    let data = clipboard.lock().unwrap();
     
    let mut keys = Vec::<String>::new();
    for key in data.keys() {
        keys.push(key.to_string());
    }
    Json(keys)
}

#[post(
    "/set_clipboard/<hostname>",
    format = "application/json",
    data = "<contents>"
)]
fn set_clipboard(
    contents: Json<UnifiedClipboard>,
    clipboard: &State<Mutex<HashMap<String, UnifiedClipboard>>>,
    hostname: String,
) -> Status {    
    let clipboards = &mut clipboard.lock().unwrap();

    // is this the best status code? not sure
    if !clipboards.contains_key(&hostname) {
        return Status::NotFound
    }
    
    let this_clip = clipboards.get_mut(&hostname).unwrap();
    
    if this_clip.passphrase.is_empty() || this_clip.passphrase == contents.passphrase {
        *this_clip = UnifiedClipboard {contents: String::from(&contents.contents),
				       passphrase: String::from(&this_clip.passphrase)};
        Status::Ok
    } else {
        Status::Forbidden
    }
}

#[post(
    "/create_clipboard/<hostname>",
    format = "application/json",
    data = "<new_clipboard>"
)]
fn create_clipboard(
    new_clipboard: Json<UnifiedClipboard>,
    clipboards: &State<Mutex<HashMap<String, UnifiedClipboard>>>,
    hostname: String,
) -> Status {
    let mut data = clipboards.lock().unwrap();
    data.insert(
        hostname,
        UnifiedClipboard {
            contents: String::from(&new_clipboard.contents),
            passphrase: String::from(&new_clipboard.passphrase)
        },
    );    
    Status::Ok
}

#[get("/")]
pub fn index(clipboard: &State<Mutex<HashMap<String, UnifiedClipboard>>>) -> Template {
    let data = clipboard.lock().unwrap();

    Template::render(
        "index",
        &TemplateContext {
            clipboards: data.clone(),
        },
    )
}

#[launch]
fn rocket() -> rocket::Rocket<rocket::Build> {
    let mut hmap = HashMap::<String, UnifiedClipboard>::new();
    
    hmap.insert(
        "server_clipboard".to_string(),
        UnifiedClipboard {
            contents: String::from("Initial contents"),
            passphrase: String::from(""),
        },
    );    

    rocket::build()
        .manage(Mutex::new(hmap))
        .mount("/", routes![index])
        .mount(
            "/api",
            routes![
                set_clipboard,
                get_clipboards,
                get_clipboard,
                kill_clipboard,
                create_clipboard
            ],
        )
        .mount("/res", FileServer::from("static"))
        .attach(Template::fairing())
}
