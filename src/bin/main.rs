extern crate ctrlc;
extern crate hello;
use hello::ThreadPool;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::sync::{Mutex, Arc};
use std::process;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("invalid port binding");
    let pool = ThreadPool::new(4);
    let m = Arc::new(Mutex::new(true));

    let looper = m.clone();
    ctrlc::set_handler(move || { 
        let mut value = looper.lock().expect("looper mutex was in use when accessed by ctrlc handler");
        println!("Shutting down.");
        *value = false;
    }).expect("Error setting Ctrl+C handler");

    
    for stream in listener.incoming() {
        let a = m.clone();
        let value = a.lock().expect("value mutex was in use when accessed by for loop");
        if *value == false{
            break;
        }
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
    drop(pool);
    process::exit(0);
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).expect("error reading stream");

    let get = b"GET / HTTP/1.1\r\n";
    let home = b"GET /home HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) || buffer.starts_with(home) {    
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let mut file = File::open(filename).expect("error opening HTML file");

    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("error reading HTML file to string");

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).expect("error writing to stream");
    stream.flush().expect("error flushing stream");
}
