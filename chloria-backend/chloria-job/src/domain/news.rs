use rand::distr::{Alphanumeric, SampleString};

const ARTICLE_ID_LENGTH: usize = 16;

pub(crate) struct NewsEntity {
    pub(crate) article_id: String, // Unique article ID to distinguish this news from others
}

impl NewsEntity {
    pub(crate) fn new(article_id: Option<String>) -> Self {
        Self {
            article_id: article_id.unwrap_or(Alphanumeric.sample_string(&mut rand::rng(), ARTICLE_ID_LENGTH)),
        }
    }
}
