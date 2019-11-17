pub mod generate_www {
    use std::sync::{Arc, Mutex};

    use plotlib::page::Page;
    use plotlib::scatter;
    use plotlib::scatter::Scatter;
    use plotlib::style::{Marker, Point};
    use plotlib::view::ContinuousView;

    use crate::twitter_stream::twitter_stream::SentimentCount;

    fn make_plot() -> String {
        // Scatter plots expect a list of pairs
        let data1 = [
            (-3.0, 2.3),
            (-1.6, 5.3),
            (0.3, 0.7),
            (4.3, -1.4),
            (6.4, 4.3),
            (8.5, 3.7),
        ];

        // We create our scatter plot from the data
        let s1: Scatter = Scatter::from_slice(&data1).style(
            scatter::Style::new()
                .marker(Marker::Square) // setting the marker to be a square
                .colour("#DD3355"),
        ); // and a custom colour

        // We can plot multiple data sets in the same view
        let data2 = [(-1.4, 2.5), (7.2, -0.3)];
        let s2: Scatter = Scatter::from_slice(&data2).style(
            scatter::Style::new()
                .colour("#35C788"),
        ); // and a different colour

        // The 'view' describes what set of data is drawn
        let v = ContinuousView::new()
            .add(&s1)
            .add(&s2)
            .x_range(-5., 10.)
            .y_range(-2., 6.)
            .x_label("Some varying variable")
            .y_label("The response of something");

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

    pub fn generate_www(sentiment: Arc<Mutex<SentimentCount>>) -> String {
        let contents_begin = r#"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Hello!</title>
</head>
<body>
<h1>Hello!</h1>
Hi from Rust.  Tweets counted so far:
"#;
        let contents_end = r#"
</body>
</html>
"#;

        let total;
        {
            let sc = sentiment.lock().unwrap();
            total = sc.total.clone();
        }

        let plot = make_plot();

        format!("{}{}\r\n{}{}", contents_begin, total, plot, contents_end)
    }
}