use std::{fs, thread};
use std::sync::{Arc, Mutex};

use twitter_stream::Token;
use yaml_rust::YamlLoader;

struct SentimentCount {
    temp: i32,
}

impl SentimentCount {
    fn new() -> SentimentCount {
        SentimentCount {
            temp: 0
        }
    }
}

fn main() {
    let sc = Arc::new(Mutex::new(SentimentCount::new()));
    let backend_handle = thread::spawn(|| {
        backend(sc)
    });

    backend_handle.join().unwrap();
}

fn backend(sentiment_count: Arc<Mutex<SentimentCount>>) {
    let tokens = load_twitter_tokens("twitter_tokens.yaml");
}

fn load_twitter_tokens(filename: &str) -> Token {
    let contents = fs::read_to_string(filename).unwrap();
    let yaml = &YamlLoader::load_from_str(&contents).unwrap()[0];
    Token::new(yaml["consumer_key"].as_str().unwrap().to_string(),
               yaml["consumer_secret"].as_str().unwrap().to_string(),
               yaml["access_key"].as_str().unwrap().to_string(),
               yaml["access_secret"].as_str().unwrap().to_string())
}