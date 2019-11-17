pub mod generate_www {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use plotlib::histogram::{Histogram, Style};
    use plotlib::page::Page;
    use plotlib::style::Bar;
    use plotlib::view::ContinuousView;

    use crate::SentimentData;

    fn make_plot(data: Vec<f64>, bin_size: usize, fill_color: &str) -> String {
        let h = Histogram::from_slice(data.as_slice(), plotlib::histogram::Bins::Count(bin_size))
            .style(Style::new().fill(fill_color));

        let v = ContinuousView::new()
            .add(&h)
            .x_label("Sentiment Score")
            .y_label("Number of Tweets");

        // A page with a single view is then saved to an SVG file
        match Page::single(&v).dimensions(800,300).to_svg() {
            Err(e) => {
                eprintln!("Error generating plot: {}", e);
                String::new()
            },
            Ok(svg) => {
                svg.to_string()
            }
        }
    }

    pub fn generate_www(sentiment_data: Arc<Mutex<SentimentData>>) -> String {
        let start_time = Utc::now().timestamp_millis();

        let html_head = String::from(r#"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Hello!</title>
</head>
<body>
"#);
        let html_foot = r#"
</body>
</html>
"#;

        let total_tweets;
        let mut all_sentiment_data;
        let mut edge_sentiment_data;  // greater than ±5 sentiment
        {
            let sd = sentiment_data.lock().unwrap();
            total_tweets = sd.total_tweets;

            all_sentiment_data = Vec::with_capacity(sd.tweets.len());
            edge_sentiment_data = Vec::with_capacity(sd.tweets.len());
            for tweets in &sd.tweets {
                all_sentiment_data.push(tweets.sentiment);

                if tweets.sentiment > 5.0 || tweets.sentiment < -5.0 {
                    edge_sentiment_data.push(tweets.sentiment);
                }
            }
        }

        let title1 = "<center><h3>Previous Hour of Twitter Sentiment Data</h3></center>";
        let plot1 = make_plot(all_sentiment_data, 15, "#228b22");

        let title2 = "<center><h3>Previous Hour of Twitter Sentiment (Greater than ±5)</h3></center>";
        let plot2 = make_plot(edge_sentiment_data, 15, "#003366");

        let hello_world = format!("Hi!  I'm written entirely in Rust.  I take Twitter data, do basic sentiment analysis on it, and then plot that data.  I've analyzed {} tweets so far.</ br>", total_tweets);

        let duration = (Utc::now().timestamp_millis() - start_time) as f64 / 1000.0;
        let rendered_in = format!("<p style=\"text-align:right;\">Page rendered live in: {} seconds.  ^_^</p>", duration);

        html_head +
            &hello_world +
            title1 +
            &plot1 +
            "<hr>" +
            title2 +
            &plot2 +
            &rendered_in +
            html_foot
    }
}