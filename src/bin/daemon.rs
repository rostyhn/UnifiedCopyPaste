extern crate x11_clipboard;
extern crate daemonize;

use std::fs::File;

use x11_clipboard::Clipboard;

use std::collections::HashMap;
use daemonize::Daemonize;
use structopt::StructOpt;

use serde::{Serialize,Deserialize};

#[derive(Debug, StructOpt, Serialize, Deserialize)]
struct Args {
    #[structopt(short,long, default_value="http://localhost")]
    url: String,
    #[structopt(short,long, default_value="8000")]
    port: String,
    #[structopt(short,long)]
    kill: bool,
    #[structopt(short,long, default_value="")]
    hostname: String
}

fn sanitize_input(opt: &mut Args) {
    
    if !opt.url.starts_with("http://") && !opt.url.starts_with("https://") {
	opt.url.insert_str(0, "http://");
    }

    if opt.url.ends_with(":") {
	opt.url.pop();
    }
    
    if opt.port.starts_with(":") {
	opt.port.remove(0);
    }
}

fn main() {    
    let mut opt = Args::from_args();    
    sanitize_input(&mut opt);
        
    let stdout = File::create("daemon.out").unwrap();
    let stderr = File::create("daemon.err").unwrap();

    if opt.kill {
	let status = match std::fs::read_to_string("daemon.pid") {
	    Ok(contents) => std::process::Command::new("kill").arg(contents).status(),
	    Err(e) => Err(e) 
	};

	if status.is_err() {
            println!("Failed to stop daemon.");
	}

	let d = std::fs::read_to_string("arguments.json").expect("Unable to read arguments file.");

	let old_opt: Args = serde_json::from_str(&d).unwrap();
	let _ = kill_clipboard(&old_opt);

	//std::fs::remove_file("arguments.json").expect("Unable to remove file");
        
        std::process::exit(1)
    }
    
    let daemon = Daemonize::new()
	.stdout(stdout)
	.stderr(stderr).pid_file("daemon.pid");
    
    match daemon.start() {
        Ok(_) => println!("Started daemon!"),
        Err(e) => eprintln!("ERROR: {}", e),
    };

    if opt.hostname.trim().is_empty() {
	match hostname::get() {
	    Ok(host) => {
		opt.hostname = host.into_string().unwrap();
		let _ = post_clipboard(&String::from("Initial String"), &opt);
		dbg!("Posted clipboard for the first time.");

		let j = match serde_json::to_string(&opt) {
		    Ok(j) => Ok(j),
		    Err(e) => Err(e)
		};

		dbg!(&j);
		
		if j.is_ok() {
		    let arguments_json = j.unwrap();
		    dbg!(&arguments_json);

		    let mut curr_dir = std::env::current_exe().unwrap();
                    curr_dir.pop();

		    std::env::set_current_dir(&curr_dir).expect("Unable to change to new directory.");
		    
		    std::fs::write("arguments.json", arguments_json).expect("Unable to write arguments file.");                    
                                       
		}
	    }
	    Err(e) => eprintln!("Error: {}", e),
	};	
    }

    let mut last = String::new();
    let clipboard = Clipboard::new().unwrap();

    // a little more complex, but removes the need to constantly hit the server,
    // plus, grabs ctrl-c input AND mouse input
    loop {
        if let Ok(curr) = clipboard.load_wait(
            clipboard.getter.atoms.primary,
            clipboard.getter.atoms.utf8_string,
            clipboard.getter.atoms.property
        ) {
            let curr = String::from_utf8_lossy(&curr);
            let curr = curr
                .trim_matches('\u{0}')
                .trim();
            if !curr.is_empty() && last != curr {
                last = curr.to_owned();                
		let _ = post_clipboard(&last.to_owned(), &opt);
            }
        }        
    }
}

fn kill_clipboard(opt: &Args) -> Result<(), reqwest::Error> {
    let mut url_string = String::from(&opt.url);
    url_string.push(':');
    url_string.push_str(&opt.port);
    url_string.push_str("/api/kill_clipboard/");
    url_string.push_str(&opt.hostname);
    
    let _res = reqwest::blocking::Client::new().post(url_string).send()?;    
    Ok(())
}

fn post_clipboard(contents: &String, opt: &Args) -> Result<(), reqwest::Error> {    
    let mut map = HashMap::new();
    map.insert("text", contents);    

    let mut url_string = String::from(&opt.url);
    url_string.push(':');
    url_string.push_str(&opt.port);
    url_string.push_str("/api/set_clipboard/");
    url_string.push_str(&opt.hostname);
    
    let _res = reqwest::blocking::Client::new().post(url_string).json(&map).send()?;    
    Ok(())
}

