use derive_getters::Getters;

#[derive(Getters)]
struct NewsFeed {
    name: String,
    url: String,
    category: Option<String>,
}

#[test]
fn news_feed() {
    let feed = NewsFeed {
        name: "NewPipe Blog".into(),
        url: "https://newpipe.net/blog/feeds/news.atom".into(),
        category: Some("OSS".into()),
    };

    assert_eq!(feed.name(), "NewPipe Blog");
    assert!(!feed.url().is_empty());
    assert!(feed.category().is_some());
}
