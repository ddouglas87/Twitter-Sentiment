use std::{fs, thread};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use crate::generate_www::generate_www::generate_www;
use crate::threadpool::threadpool::ThreadPool;
use crate::twitter_stream::twitter_stream::SentimentCount;
use crate::twitter_stream::twitter_stream::twitter_stream;

mod threadpool;
mod twitter_stream;
mod generate_www;


fn main() {
    let sc = Arc::new(Mutex::new(SentimentCount::new()));

    let sc_clone = sc.clone();
    let backend_handle = thread::spawn(|| {
        twitter_stream(sc_clone)
    });

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(2);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let sc_clone = sc.clone();
        pool.execute(move || {
            let mut buffer = [0; 512];
            stream.read(&mut buffer).unwrap();

            //    println!("Request: {}", String::from_utf8_lossy(&buffer));

            let (status_line, contents) = if !buffer.starts_with(b"GET / HTTP/1.1") {
                let status_line = "HTTP/1.1 404 NOT FOUND";
                let contents = fs::read_to_string("www/404.htm").unwrap();
                (status_line, contents)
            } else {
                let status_line = "HTTP/1.1 200 OK";
                let contents = generate_www(sc_clone);
                (status_line, contents)
            };

            let response = format!("{}\r\n\r\n{}", status_line, contents);

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        });
    }

    backend_handle.join().unwrap();
}
