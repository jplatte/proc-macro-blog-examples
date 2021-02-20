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

#[derive(Getters)]
struct NewsFeedRef<'a> {
    name: &'a str,
    url: &'a str,
    category: Option<&'a str>,
}

#[test]
fn news_feed_ref() {
    let feed = NewsFeedRef {
        name: "Sudo Satirical",
        url: "https://sudosatirical.com/articles/index.xml",
        category: None,
    };

    assert_eq!(*feed.name(), "Sudo Satirical");
    assert!(!feed.url().is_empty());
    assert!(feed.category().is_none());
}
