use std::io::prelude::*;
use std::thread;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str::from_utf8;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::io::BufReader;

fn main() {
    let ip = "127.0.0.1:8888";
    let listener = TcpListener::bind(ip).unwrap();
    let total_reqs = Arc::new(Mutex::new(0));
    let valid_reqs = Arc::new(Mutex::new(0));
    println!("Server started at {}", ip);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let total_reqs = total_reqs.clone();
                let valid_reqs = valid_reqs.clone();
                thread::spawn(move || {
                    handle_client(stream, total_reqs, valid_reqs);
                });
            },
            Err(e) => {println!("Error: {}", e);}
        }
        {
            let total_reqs = total_reqs.lock().unwrap();
            //println!("Total Requests: {}", *total_reqs);
        }
        {
            let valid_reqs = valid_reqs.lock().unwrap();
            //println!("Valid Requests: {}", *valid_reqs);
        }
    }
}

fn handle_client(mut stream: TcpStream, total_reqs: Arc<Mutex<u64>>, valid_reqs: Arc<Mutex<u64>>) {
    match stream.peer_addr() {
        Ok(address) => {
            //println!("Address: {}", address);
            {
                let mut total_reqs = total_reqs.lock().unwrap();
                *total_reqs += 1;
            }

            let mut buf = vec![0 ; 500];

            let _ = stream.read(&mut buf); //TODO check output

            let s = from_utf8(&mut buf).unwrap();
            let split = s.split("\n");
            let lines = split.collect::<Vec<&str>>();

            let split2 = lines[0].split_whitespace();

            let words = split2.collect::<Vec<&str>>();
            //for w in &words {
            //    println!("{}", w);
            //}

            if words.len() < 1 {return;}
            for w in &words {
                println!("{}", w);
            }
            //let mut f = String::from(words[1]);
            let mut multFiles = words[1].split("/").collect::<Vec<&str>>();

            multFiles.remove(0);
            //let valid_reqs = Arc::new(Mutex::new(0));

            let header = "<br>HTTP/1.1\nContent-Type: text/html; charset=UTF-8\n\n<html>\n";
            let _ = stream.write(header.as_bytes());
            let thisStream = Arc::new(Mutex::new(stream));
            for f in multFiles {
                let file = String::from(f);
                let thisStream = thisStream.clone();
                thread::spawn(move || {
                    handle_file(thisStream, &*file, &address);
                });
            }

        },
        Err(e) => {println!("Error: {}", e);}
    };

}

fn handle_file(mut thisStream: Arc<Mutex<TcpStream>>, file: &str, address : &std::net::SocketAddr) {
    if file.contains("..") {
        let to_write = format!(//"<br>HTTP/1.1 403 Forbidden\nContent-Type: text/html; charset=UTF-8\n\n<html>\n
        "<body>\n<h1>403 Forbidden</h1>\nClient address: {add}<br>\nRequested file: {file}<br>\n</body>
        \n</html><br>",add=address, file=file);
        let mut thisStream = thisStream.lock().unwrap();
        let _ = thisStream.write(to_write.as_bytes());
    }
    else {
        //println!{"{}, ", file}
        if file.len() < 1 {return;}
        // removes backslash
        //file.remove(0);
        println!("Thread spawned: File: {}", file);
        match File::open(file) {
            Ok(f) => {
                //let mut valid_reqs = valid_reqs.lock().unwrap();
                //*valid_reqs += 1; //TODO should this be here?

                let mut write_buf = String::new();
                let mut reader = BufReader::new(f);
                match reader.read_to_string(&mut write_buf) {
                    Ok(_) => {},
                    Err(e) => {println!("Error: {}", e)},
                }
                let mut thisStream = thisStream.lock().unwrap();
                let _ = thisStream.write(write_buf.as_bytes());
                let _ = thisStream.write("<br>\n".as_bytes());


        },
            Err(_) => {
                let to_write = format!(//"<br>HTTP/1.1 404 Not Found\nContent-Type: text/html; charset=UTF-8\n\n<html>
                "\n<body>\n<h1>404 Not Found</h1>\nClient address: {add}<br>\nRequested file: {file}<br>\n</body>
                \n</html><br>\n",add=address, file=file);

                let mut thisStream = thisStream.lock().unwrap();
                let _ = thisStream.write(to_write.as_bytes());
            }
        }
    }


}


// things to keep in mind:
        // step 1: unsure if formatting is correct, may have to split on
        // %0A/ rather than by newline and then / based on the way httperf
        // translates to this
        // step 2: files can be written as they are processed! order doesn't matter
        // TODO check up on 'RUST_BACKTRACE=1'





// sources:
// https://steemit.com/rust-series/@jimmco/rust-lang-series-episode-33-tcp-client-rust-series
