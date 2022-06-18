extern crate daemonize;
extern crate x11_clipboard;

//use std::fs::File;
//use daemonize::Daemonize;
use x11_clipboard::Clipboard;

use std::collections::HashMap;
use structopt::StructOpt;
use single_instance::SingleInstance;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, StructOpt, Serialize, Deserialize)]
struct Args {
    #[structopt(short, long, default_value = "http://localhost")]
    url: String,
    #[structopt(short, long, default_value = "8000")]
    port: String,
    #[structopt(short, long, default_value = "")]
    hostname: String,
    #[structopt(long, default_value = "")]
    passphrase: String
}

fn main() {

    let my_instance = SingleInstance::new("ucp-daemon").unwrap();

    if my_instance.is_single() {

        let mut opt = Args::from_args();
        sanitize_input(&mut opt);
    
        if opt.hostname.trim().is_empty() {
            let hostname = hostname::get().expect("Failed to retrieve hostname.");
            opt.hostname = hostname.into_string().unwrap();
        }

        let closure_opt = opt.clone();
        let client = reqwest::blocking::Client::new();

        let _ = ctrlc::set_handler(move || {
            match kill_clipboard(&closure_opt) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);            
                }
            }
            std::process::exit(-1);        
        });

        match create_clipboard(&opt, &client) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(-1);
            }
        };   
   
        let mut last = String::new();
        let clipboard = Clipboard::new().unwrap();
        
        loop {
            // check if clipboard changed OR if named pipe has input...
            // that would figure everything
            // or... what if there were multiple threads all doing this

            // main communicator thread - listens to input from server & pings it occassionally
            // one thread that listens on input from clipboard -> if detected, send to main to send to server
        
            if let Ok(curr) = clipboard.load_wait(
                clipboard.getter.atoms.primary,
                clipboard.getter.atoms.utf8_string,
                clipboard.getter.atoms.property,
            ) {
                let curr = String::from_utf8_lossy(&curr);
                let curr = curr.trim_matches('\u{0}').trim();

                // dies when the user tries to copy paste something and the server is dead
                if !curr.is_empty() && last != curr {
                    last = curr.to_owned();
                    match post_clipboard(&last.to_owned(), &opt, &client) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("{}", e);
                            std::process::exit(-1);
                        }
                    };
                }
            }

            // read from named pipe & paste into clipboard
            // have either the server automatically send changes to the pipe
            // OR have the user have set up a script that grabs from the pipe and pastes into it
        }
    } else {
        eprintln!("Daemon is already running");
        std::process::exit(-1);
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
    
    reqwest::blocking::Client::new()
        .post(url)
        .json(&map)
        .send()?;
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
