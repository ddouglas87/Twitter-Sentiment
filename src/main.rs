use std::{env, fs, thread};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use crate::generate_www::generate_www::generate_www;
use crate::threadpool::threadpool::ThreadPool;
use crate::twitter_stream::twitter_stream::Tweet;
use crate::twitter_stream::twitter_stream::twitter_stream;

mod threadpool;
mod twitter_stream;
mod generate_www;

#[derive(Debug)]
pub struct SentimentData {
    total_tweets: usize,
    tweets: VecDeque<Tweet>,
}

const TWEETS_A_SECOND:i64 = 60;
const TWEET_RETAINMENT_SECONDS:i64 = 3*60;

fn main() {
    let sentiment_data = Arc::new(Mutex::new(SentimentData {
        total_tweets: 0,
        tweets: VecDeque::with_capacity((TWEETS_A_SECOND * TWEET_RETAINMENT_SECONDS) as usize),
    }));

    let sd_clone = sentiment_data.clone();
    let backend_handle = thread::spawn(|| {
        twitter_stream(sd_clone)
    });

    let port = env::var("PORT").unwrap_or_else(|_| { String::from("7878") });
    let listener = TcpListener::bind(String::from("0.0.0.0:") + &port).unwrap();
    let pool = ThreadPool::new(num_cpus::get());

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let sd_clone = sentiment_data.clone();
        pool.execute(move || {
            let mut buffer = [0; 512];
            stream.read(&mut buffer).unwrap();

            //    println!("Request: {}", String::from_utf8_lossy(&buffer));

            let (status_line, contents) = if !buffer.starts_with(b"GET / HTTP/1.1") {
                let status_line = "HTTP/1.1 404 NOT FOUND";
                let contents = fs::read_to_string("www/404.html").unwrap();
                (status_line, contents)
            } else {
                let status_line = "HTTP/1.1 200 OK";
                let contents = generate_www(sd_clone);
                (status_line, contents)
            };

            let response = format!("{}\r\n\r\n{}", status_line, contents);

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        });
    }

    backend_handle.join().unwrap();
}
