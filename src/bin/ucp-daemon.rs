use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use x11_clipboard::Clipboard;
use structopt::StructOpt;
use single_instance::SingleInstance;
use serde::{Deserialize, Serialize};

use websocket::client::ClientBuilder;
use websocket::{OwnedMessage, Message};

#[derive(Clone, Debug, StructOpt, Serialize, Deserialize)]
struct Args {
    // domain name only
    #[structopt(short, long, default_value = "localhost")]
    domain: String,
    #[structopt(short, long, default_value = "8000")]
    port: String,
    // identifies the current daemon
    #[structopt(short, long, default_value = "")]
    hostname: String,
    #[structopt(long, default_value = "")]
    passphrase: String,
    #[structopt(long, default_value = "")]
    url: String,
    #[structopt(short, long)]
    ssl_enabled: bool
}

fn main() -> ! {

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
   
        
        loop {
            let ws_str = build_ws_str(&opt);

            // this can be split into two!
            let client = ClientBuilder::new(&ws_str)
                .unwrap()
                .connect_insecure()
                .unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();
            
            let (tx, rx) = mpsc::channel();

            // reciever's channel to send back messages
            let tx_1 = tx.clone();

            // clipboard listener thread
            thread::spawn(move || {
                let mut last = String::new();
                let clipboard = Clipboard::new().unwrap();
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
                            match tx.send(Message::text(last.to_owned())) {
                                Ok(_) => (),
                                Err(e) => {
                                    eprintln!("Error: {}", e);
                                    std::process::exit(-1)
                                }
                            };
                        }
                    }
                }
            });

            // websocket sender thread
            thread::spawn(move || {
                loop {
                    let msg = match rx.recv() {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(-1)
                        }
                    };
                    
                    let res = sender.send_message(&msg);
                    match res {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(-1)
                        }
                    }
                }
            });

            
            // server listener thread - main
            loop {
                let clipboard = Clipboard::new().unwrap();                
                for message in receiver.incoming_messages() {
                    let msg = match message {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("Error: {}. Server stopped responding.", e);
                            std::process::exit(-1)
                        }
                    };

                    match msg {
                        OwnedMessage::Ping(d) => {
                            match tx_1.send(Message::pong(d)) {
                                Ok(()) => (),
                                Err(e) => {
                                    eprintln!("Error: {}.", e);
                                    std::process::exit(-1)
                                }                                    
                            };
                        },
                        OwnedMessage::Pong(d) => {
                            match tx_1.send(Message::ping(d)) {
                                Ok(()) => (),
                                Err(e) => {
                                    eprintln!("Error: {}.", e);
                                    std::process::exit(-1)
                                }                                    
                            };
                        },
                        OwnedMessage::Close(_) => {
                            let _ = tx_1.send(Message::close());                            
                        },                                                    
                        OwnedMessage::Text(d) => {
                            let d = d.trim_matches('\\').trim();
                            let d = d.trim_matches('\"').trim();
                            let d = d.trim_matches('\u{0}').trim();
                            
                            match clipboard.store(clipboard.getter.atoms.primary, clipboard.getter.atoms.string, d) {
                                Ok(_) => (),
                                Err(e) => eprintln!("{}", e)
                            };
                        },
                        _ => {
                            eprintln!("Message type not supported.");
                            std::process::exit(-1)
                        }
                    };
                }
            }            
        }
    } else {
        eprintln!("Daemon is already running");
        std::process::exit(-1);
    }
}

fn sanitize_input(opt: &mut Args) {        
    opt.url.insert_str(0, &get_http_str(opt.ssl_enabled));
    opt.url.push_str(&opt.domain);         
}

fn build_url(opt: &Args, url_path: &String) -> String {
    let mut url_string = String::from(&opt.url);
    url_string.push(':');
    url_string.push_str(&opt.port);
    url_string.push_str(url_path);
    url_string.push_str(&opt.hostname);

    url_string
}

fn get_http_str(ssl_enabled: bool) -> String {
    if ssl_enabled {
       "https://".to_string() 
    } else {
        "http://".to_string()
    }   
}

fn get_ws_str(ssl_enabled: bool) -> String {
    if ssl_enabled {
        "wss://".to_string()
    } else {
        "ws://".to_string()
    }
}

fn build_ws_str(opt: &Args) -> String {
    let mut ws_str = String::from(&opt.domain);
    ws_str.insert_str(0, &get_ws_str(opt.ssl_enabled));
    ws_str.push(':');
    ws_str.push_str(&opt.port);
    ws_str.push_str("/api/clipboard_websocket/");
    ws_str.push_str(&opt.hostname);
    
    ws_str
}

/* All HTTP requests */
    
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

/*
#[deprecated]
fn set_clipboard(
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
}*/
