extern crate x11_clipboard;
extern crate daemonize;

use std::fs::File;
use x11_clipboard::Clipboard;

use std::collections::HashMap;
use daemonize::Daemonize;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(short,long, default_value="http://localhost")]
    url: String,
    #[structopt(short,long, default_value="8000")]
    port: String,
}

fn sanitize_input(opt: &mut Args) {
    
    if !opt.url.starts_with("http://") && !opt.url.starts_with("https://") {
	opt.url.insert_str(0,"http://");
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
    
    let stdout = File::create("tmp/daemon.out").unwrap();
    let stderr = File::create("tmp/daemon.err").unwrap();
    
    let daemon = Daemonize::new()
	.stdout(stdout)
	.stderr(stderr);
    
    match daemon.start() {
        Ok(_) => println!("Started daemon!"),
        Err(e) => eprintln!("ERROR: {}", e),
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

fn post_clipboard(contents: &String, opt: &Args) -> Result<(), reqwest::Error> {    
    let mut map = HashMap::new();
    map.insert("text", contents);
    let mut url_string = String::from(&opt.url);
    url_string.push(':');
    url_string.push_str(&opt.port);
    url_string.push_str("/api/set_clipboard");
    
    let res = reqwest::blocking::Client::new().post(url_string).json(&map).send()?;    
    Ok(())
}

