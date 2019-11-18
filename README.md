# Twitter-Sentiment
Takes tweets, performs a very basic sentiment analysis on them, plots this data, and serves this web page.

### About
This project is my Hello World to Rust.  It's a fun project, to play with, but it isn't designed to be used in any serious capacity.

### How to Use

1) Create an app at https://apps.twitter.com/  (Requires an account created at https://developer.twitter.com/)
2) Create a twitter_tokens.yaml file in the base folder and make its contents look something like this:
```
   consumer_key: ...
   consumer_secret: ...
   access_key: ...
   access_secret: ...
```
3) git clone this repo, cd to it, cargo run
4) Open: http://localhost:7878