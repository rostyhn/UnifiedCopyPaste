extern crate daemonize;
extern crate x11_clipboard;

use std::fs::File;

use x11_clipboard::Clipboard;

use daemonize::Daemonize;
use std::collections::HashMap;
use structopt::StructOpt;

use serde::{Deserialize, Serialize};



#[derive(Debug, StructOpt, Serialize, Deserialize)]
struct Args {
    #[structopt(short, long, default_value = "http://localhost")]
    url: String,
    #[structopt(short, long, default_value = "8000")]
    port: String,
    #[structopt(short, long)]
    kill: bool,
    #[structopt(short, long, default_value = "")]
    hostname: String,
    #[structopt(long, default_value = "")]
    passphrase: String
}

fn main() {
    let mut opt = Args::from_args();
    sanitize_input(&mut opt);

    let stdout = File::create("daemon.out").unwrap();
    let stderr = File::create("error.log").unwrap();

    if opt.kill {
        let status = match std::fs::read_to_string("daemon.pid") {
            Ok(contents) => std::process::Command::new("kill").arg(contents).status(),
            Err(e) => panic!("Error opening PID file: {:?}", e),
        };

        println!("{:?}", status);

        let d = std::fs::read_to_string("arguments.json").expect("Unable to read arguments file.");

        let old_opt: Args = serde_json::from_str(&d).unwrap();

        match kill_clipboard(&old_opt) {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e),
        };

        std::fs::remove_file("arguments.json").expect("Unable to remove file.");
        std::fs::remove_file("daemon.pid").expect("Unable to remove file.");

        std::process::exit(1)
    }

    let daemon = Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .pid_file("daemon.pid");

    match daemon.start() {
        Ok(_) => println!("Started daemon!"),
        Err(e) => eprintln!("Error starting daemon: {}", e),
    };

    if opt.hostname.trim().is_empty() {
        let hostname = hostname::get().expect("Failed to retrieve hostname.");
        opt.hostname = hostname.into_string().unwrap();

        let arguments_json =
            serde_json::to_string(&opt).expect("Failed to convert hostname to JSON.");
        // hostname command moves us into /, so we need to return to the executable's directory
        let mut curr_dir = std::env::current_exe().unwrap();
        curr_dir.pop();
        std::env::set_current_dir(&curr_dir).expect("Unable to change to new directory.");

        match std::fs::write("arguments.json", arguments_json) {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        };
    }

    let client = reqwest::blocking::Client::new();

    match create_clipboard(&opt, &client) {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e),
    };    
    
    let mut last = String::new();
    let clipboard = Clipboard::new().unwrap();

    // a little more complex, but removes the need to constantly hit the server.
    // plus, grabs ctrl-c input AND mouse input
    loop {
        if let Ok(curr) = clipboard.load_wait(
            clipboard.getter.atoms.primary,
            clipboard.getter.atoms.utf8_string,
            clipboard.getter.atoms.property,
        ) {
            let curr = String::from_utf8_lossy(&curr);
            let curr = curr.trim_matches('\u{0}').trim();          
            
            if !curr.is_empty() && last != curr {
                last = curr.to_owned();
                match post_clipboard(&last.to_owned(), &opt, &client) {
                    Ok(_) => (),
                    Err(e) => eprintln!("{}", e),
                };
            }
        }
    }
}

fn sanitize_input(opt: &mut Args) {
    if !opt.url.starts_with("http://") && !opt.url.starts_with("https://") {
        opt.url.insert_str(0, "http://");
    }

    if opt.url.ends_with(':') {
        opt.url.pop();
    }

    if opt.port.starts_with(':') {
        opt.port.remove(0);
    }
}

fn build_url(opt: &Args, url_path: &String) -> String {

    let mut url_string = String::from(&opt.url);
    url_string.push(':');
    url_string.push_str(&opt.port);
    url_string.push_str(url_path);
    url_string.push_str(&opt.hostname);

    url_string
    
}

fn kill_clipboard(opt: &Args) -> Result<(), reqwest::Error> {

    let mut map = HashMap::new();
    map.insert("passphrase", &opt.passphrase);

    let url = build_url(opt, &"/api/kill_clipboard/".to_string());    
    
    reqwest::blocking::Client::new().post(url).json(&map).send()?;
    Ok(())
}

fn create_clipboard(opt: &Args, client: &reqwest::blocking::Client) -> Result<(), reqwest::Error> {
    let mut map = HashMap::new();
    map.insert("contents", "Initial contents");
    map.insert("passphrase", &opt.passphrase);

    let url = build_url(opt, &"/api/create_clipboard/".to_string());

    client.post(url).json(&map).send()?;    
    
    Ok(())
}

fn post_clipboard(
    contents: &String,    
    opt: &Args,
    client: &reqwest::blocking::Client,
) -> Result<(), reqwest::Error> {
    
    let mut map = HashMap::new();
    map.insert("contents", contents);
    map.insert("passphrase", &opt.passphrase);
    
    let url = build_url(opt, &"/api/set_clipboard/".to_string());
    
    client.post(url).json(&map).send()?;

    Ok(())
}
