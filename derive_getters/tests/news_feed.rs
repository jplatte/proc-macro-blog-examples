use derive_getters::Getters;

#[derive(Getters)]
struct NewsFeed {
    name: String,
    url: String,
    category: Option<String>,
}
