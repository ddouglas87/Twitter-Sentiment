use std::{fs, thread};
use std::sync::{Arc, Mutex};

use twitter_stream::{Token, TwitterStreamBuilder};
use twitter_stream::rt::{self, Future, Stream};
use yaml_rust::YamlLoader;
use rustc_serialize::json;
use chrono::{ParseResult, DateTime, FixedOffset};

/// https://developer.twitter.com/en/docs/tweets/data-dictionary/overview/tweet-object
struct Tweet {
    created_at: ParseResult<DateTime<FixedOffset>>,
    text: String,
}

#[derive(Debug)]
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

    let sc_clone = sc.clone();
    let backend_handle = thread::spawn(|| {
        backend(sc_clone)
    });

    backend_handle.join().unwrap();
}

/// Single threaded, due to sentiment analysis not needing a lot of processing power.
fn backend(sentiment_count: Arc<Mutex<SentimentCount>>) {
    let tokens = load_twitter_tokens("twitter_tokens.yaml");
    let keywords = "twitter, facebook, google, travel, art, music, photography, love, fashion, food";

    let future = TwitterStreamBuilder::filter(tokens)
        .track(Some(keywords))
        .listen()
        .unwrap()
        .flatten_stream()
        .for_each(|json| {
            let tweet = json_decode(json.to_string());
            Ok(())
        })
        .map_err(|e| eprintln!("error: {}", e));

    rt::run(future);
}

fn json_decode(json: String) -> Option<Tweet> {
    #[derive(Debug)]
    #[derive(RustcDecodable)]
    struct JSONTweet { created_at: String, text: String, lang: Option<String> }
    let decoded: json::DecodeResult<JSONTweet> = json::decode(&json);

    if let Ok(t) = decoded {
        if let Some(lang) = &t.lang {
            if lang.starts_with("en") {
                return Some(Tweet {
                    created_at: DateTime::parse_from_str(&t.created_at, "%a %b %d %H:%M:%S %z %Y"),
                    text: t.text,
                })
            }
        }
    }

    None
}

fn load_twitter_tokens(filename: &str) -> Token {
    let contents = fs::read_to_string(filename).unwrap();
    let yaml = &YamlLoader::load_from_str(&contents).unwrap()[0];
    Token::new(yaml["consumer_key"].as_str().unwrap().to_string(),
               yaml["consumer_secret"].as_str().unwrap().to_string(),
               yaml["access_key"].as_str().unwrap().to_string(),
               yaml["access_secret"].as_str().unwrap().to_string())
}