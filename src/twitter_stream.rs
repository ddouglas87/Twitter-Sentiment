pub mod twitter_stream {
    use std::fs;
    use std::sync::{Arc, Mutex};

    use chrono::{DateTime, FixedOffset, ParseResult};
    use rustc_serialize::json;
    use twitter_stream::{Token, TwitterStreamBuilder};
    use twitter_stream::rt::{self, Future, Stream};
    use yaml_rust::YamlLoader;


    #[derive(Debug)]
    pub struct SentimentCount {
        pub total: i32,
    }

    impl SentimentCount {
        pub fn new() -> SentimentCount {
            SentimentCount {
                total: 0
            }
        }
    }

    #[derive(Debug)]
    #[derive(RustcDecodable)]
    struct JSONTweet { created_at: String, text: String, lang: Option<String> }

    /// https://developer.twitter.com/en/docs/tweets/data-dictionary/overview/tweet-object
    #[derive(Debug, Clone)]
    struct Tweet {
        created_at: ParseResult<DateTime<FixedOffset>>,
        text: String,
        sentiment: i32,
    }


    /// Single threaded, due to sentiment analysis not needing a lot of processing power.
    pub fn twitter_stream(sentiment_count: Arc<Mutex<SentimentCount>>) {
        let tokens = load_twitter_tokens("twitter_tokens.yaml");
        let keywords = "twitter, facebook, google, travel, art, music, photography, love, fashion, food";

        // rrb-tree
//    let mut vec = Vector::new();

        let mut highest = 0.0;
        let mut lowest = 0.0;
        let future = TwitterStreamBuilder::filter(tokens)
            .track(Some(keywords))
            .listen()
            .unwrap()
            .flatten_stream()
            .for_each(move |json| {
                let tweet: json::DecodeResult<JSONTweet> = json::decode(&json);
                if let Err(_) = tweet { return Ok(()) }
                let tweet = tweet.unwrap();

                // Captures English tweets, because the sentiment analysis library supports English only.
                if None == tweet.lang || !tweet.lang.unwrap().starts_with("en") { return Ok(()) }

//            let datetime = DateTime::parse_from_str(&tweet.created_at, "%a %b %d %H:%M:%S %z %Y");
                let analysis = sentiment::analyze(tweet.text.clone());

                //// Document what the sentiment range is.
                if analysis.score > highest {
                    highest = analysis.score;
                    println!("\nNew highest score found!  {}\n{}", highest, tweet.text);
                }
                if analysis.score < lowest {
                    lowest = analysis.score;
                    println!("\nNew lowest score found!: {}\n{}", lowest, tweet.text);
                }
                ////

                match sentiment_count.lock() {
                    Err(e) => {
                        eprintln!("Error can not get lock on sentiment count: {}", e);
                    }
                    Ok(mut sc) => {
                        sc.total += 1;
                    }
                }

//            vec.push_back(Tweet {
//                created_at: datetime,
//                text: tweet.text,
//                sentiment: analysis.score as i32,
//            });

                Ok(())
            })
            .map_err(|e| eprintln!("error: {}", e));

        rt::run(future);
    }

    fn load_twitter_tokens(filename: &str) -> Token {
        let contents = fs::read_to_string(filename).unwrap();
        let yaml = &YamlLoader::load_from_str(&contents).unwrap()[0];
        Token::new(yaml["consumer_key"].as_str().unwrap().to_string(),
                   yaml["consumer_secret"].as_str().unwrap().to_string(),
                   yaml["access_key"].as_str().unwrap().to_string(),
                   yaml["access_secret"].as_str().unwrap().to_string())
    }
}