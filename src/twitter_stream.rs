pub mod twitter_stream {
    use std::fs;
    use std::sync::{Arc, Mutex};

    use chrono::{DateTime, Utc};
    use rustc_serialize::json;
    use twitter_stream::{Token, TwitterStreamBuilder};
    use twitter_stream::rt::{self, Future, Stream};
    use yaml_rust::YamlLoader;

    use crate::{SentimentData, TWEET_RETAINMENT_SECONDS};

    /// https://developer.twitter.com/en/docs/tweets/data-dictionary/overview/tweet-object
    #[derive(Debug, Clone)]
    pub struct Tweet {
        pub created_at: i64,
        pub text: String,
        pub sentiment: f64,
    }

    const TWITTER_TOKENS_FILENAME:&str = "twitter_tokens.yaml";

    /// Twitter Stream collects data from a collection of keywords, and saves that into the
    /// sentiment_data variable.
    pub fn twitter_stream(sentiment_data: Arc<Mutex<SentimentData>>) {
        let tokens = load_twitter_tokens(TWITTER_TOKENS_FILENAME);
        let keywords = "twitter, facebook, google, travel, art, music, photography, love, fashion, food";

        let mut highest = 0.0;
        let mut lowest = 0.0;
        let future = TwitterStreamBuilder::filter(tokens)
            .track(Some(keywords))
            .listen()
            .unwrap()
            .flatten_stream()
            .for_each(move |json| {
                // Temporary struct to store decoded json data.
                #[derive(RustcDecodable)]
                struct JSONTweet { created_at: String, text: String, lang: Option<String> }

                let tweet: json::DecodeResult<JSONTweet> = json::decode(&json);
                if let Err(_) = tweet { return Ok(()) }
                let tweet = tweet.unwrap();

                // Captures English tweets, because the sentiment analysis library supports English only.
                if None == tweet.lang || !tweet.lang.unwrap().starts_with("en") { return Ok(()) }

                // Single threaded, because it isn't very process intensive.  If it was intensive
                // this would be sent to a thread or multiple threads.
                let analysis = sentiment::analyze(tweet.text.clone());
                let datetime = DateTime::parse_from_str(&tweet.created_at, "%a %b %d %H:%M:%S %z %Y").unwrap().timestamp();

                //// Document high scores.
                if analysis.score > highest {
                    highest = analysis.score;
                    println!("\nNew highest score found!  {}\n{}", highest, tweet.text);
                }
                if analysis.score < lowest {
                    lowest = analysis.score;
                    println!("\nNew lowest score found!: {}\n{}", lowest, tweet.text);
                }
                ////

                // If this was a larger project, this data would go to a database.  Instead, it's
                // being piped into a shared data structure here.
                {
                    let mut sd = sentiment_data.lock().unwrap();

                    // Drop older tweets
                    while sd.tweets.len() > 1 {
                        let tweet = sd.tweets.get(0).unwrap();
                        if Utc::now().timestamp() - tweet.created_at > TWEET_RETAINMENT_SECONDS {
                            sd.tweets.pop_front();
//                        sd.total_tweets -= 1;
                        } else {
                            break;
                        }
                    }

                    // Add new tweet
                    sd.total_tweets += 1;
                    sd.tweets.push_back(Tweet {
                        created_at: datetime,
                        text: tweet.text,
                        sentiment: analysis.score as f64,
                    });
                }

                Ok(())
            })
            .map_err(|e| eprintln!("error: {}", e));

        rt::run(future);
    }

    fn load_twitter_tokens(filename: &str) -> Token {
        let contents = fs::read_to_string(filename).unwrap_or_else(|e| {
            panic!("Error opening twitter api keys file {}\n{}", TWITTER_TOKENS_FILENAME, e);
        });
        let yaml = &YamlLoader::load_from_str(&contents).unwrap()[0];
        Token::new(yaml["consumer_key"].as_str().unwrap().to_string(),
                   yaml["consumer_secret"].as_str().unwrap().to_string(),
                   yaml["access_key"].as_str().unwrap().to_string(),
                   yaml["access_secret"].as_str().unwrap().to_string())
    }
}