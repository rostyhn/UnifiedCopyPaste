extern crate x11_clipboard;
extern crate daemonize;

use std::fs::File;
use x11_clipboard::Clipboard;

use std::collections::HashMap;
use daemonize::Daemonize;


fn main() {    

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
		let _ = post_clipboard(last.to_owned());
            }
        }        
    }
}

fn post_clipboard(contents: String) -> Result<(), reqwest::Error> {    
    let mut map = HashMap::new();
    map.insert("text", &contents);
    let res = reqwest::blocking::Client::new().post("http://localhost:8000/api/set_clipboard").json(&map).send()?;    
    Ok(())
}

