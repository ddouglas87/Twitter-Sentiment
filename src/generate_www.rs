pub mod generate_www {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use plotlib::histogram::{Histogram, Style};
    use plotlib::page::Page;
    use plotlib::style::Bar;
    use plotlib::view::ContinuousView;

    use crate::{SentimentData, TWEET_RETAINMENT_SECONDS};

    fn make_plot(data: &Vec<f64>, bin_size: usize, fill_color: &str) -> String {
        let h = Histogram::from_slice(data, plotlib::histogram::Bins::Count(bin_size))
            .style(Style::new().fill(fill_color));

        let v = ContinuousView::new()
            .add(&h)
            .x_label("Sentiment Score")
            .y_label("Number of Tweets");

        // A page with a single view is then saved to an SVG file
        match Page::single(&v).to_svg() {
            Err(e) => {
                eprintln!("Error generating plot: {}", e);
                String::new()
            },
            Ok(svg) => {
                svg.to_string()
            }
        }
    }

    /// Makes a web page, with a plot
    /// Right now it dynamically makes the page every time, due to not being designed to take on a
    /// heavy load of users.
    pub fn generate_www(sentiment_data: Arc<Mutex<SentimentData>>) -> String {
        let start_time = Utc::now().timestamp_millis();

        let html_head = String::from(r#"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Hello!</title>
<script type="text/javascript">
function load() { setTimeout("window.open(self.location, '_self');", 1500); }
</script>
</head>
<body onload="load()">
"#);
        let html_foot = r#"
</body>
</html>
"#;

        // Copies data into a different format for the plotting software.
        let total_tweets;
        let mut all_sentiment_data;
        {
            let sd = sentiment_data.lock().unwrap();
            total_tweets = sd.total_tweets;

            all_sentiment_data = Vec::with_capacity(sd.tweets.len());
            for tweets in &sd.tweets {
                all_sentiment_data.push(tweets.sentiment);
            }
        }

        let title1 = format!("<center><h3>Previous {} Minutes of Twitter Sentiment Data</h3></center>", TWEET_RETAINMENT_SECONDS / 60);
        let plot1 = make_plot(&all_sentiment_data, 15, "#228b22");

        let hello_world = format!("Hi!  I'm written entirely in Rust.  I take Twitter data, do basic sentiment analysis on it, and then plot that data.  I've analyzed {} tweets so far.</ br>", total_tweets);

        let duration = (Utc::now().timestamp_millis() - start_time) as f64 / 1000.0;
        let rendered_in = format!("<p style=\"text-align:right;\">Page rendered live in {:.*} seconds.  ^_^</p>", 3, duration);

        html_head +
            &hello_world +
            &title1 +
            &plot1 +
            "<hr>" +
            &rendered_in +
            html_foot
    }
}