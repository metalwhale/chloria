use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use chrono_tz::Tz;
use log::{error, info};
use reqwest::Client;
use serde::Deserialize;

use crate::execution::ports::news_fetcher::{FetchNewsArticle, FetchNewsHandler, FetchNewsOutput, NewsFetcher};

// Doc: https://newsdata.io/documentation/#http_response
#[derive(Deserialize)]
struct NewsResponse {
    #[serde(rename = "totalResults")]
    total_results: u16,
    results: Vec<NewsResponseResult>,
    #[serde(rename = "nextPage")]
    next_page: Option<String>,
}

#[derive(Deserialize)]
struct NewsResponseResult {
    article_id: String,
    link: Option<String>,
    title: Option<String>,
    description: Option<String>,
    image_url: Option<String>,
    #[serde(rename = "pubDate")]
    pub_date: Option<String>,
    #[serde(rename = "pubDateTZ")]
    pub_date_tz: Option<String>,
}

pub(crate) struct NewsdataClient {
    api_key: String,
    pages_num_limit: Option<u16>,
}

impl NewsdataClient {
    pub(crate) fn new(api_key: String, pages_num_limit: Option<u16>) -> Self {
        Self {
            api_key,
            pages_num_limit,
        }
    }

    async fn fetch_page(&self, page: Option<String>) -> Result<(u16, Vec<FetchNewsArticle>, Option<String>)> {
        // Doc: https://newsdata.io/news-sources/Japan-news-api
        let mut url = format!("https://newsdata.io/api/1/latest?country=jp&apikey={}", self.api_key);
        if let Some(page) = page {
            url.push_str(&format!("&page={page}"));
        }
        let response_text = Client::new().get(url).send().await?.text().await?;
        let news_response: NewsResponse = serde_json::from_str(&response_text)
            .with_context(|| format!("Deserializing the response text: {}", response_text))?;
        let mut articles = vec![];
        for result in news_response.results.into_iter() {
            let published_time = match (result.pub_date, result.pub_date_tz) {
                (Some(pub_date), Some(pub_date_tz)) => {
                    // TODO: Handle errors instead of returning early
                    let timezone: Tz = pub_date_tz.parse()?;
                    match NaiveDateTime::parse_from_str(&pub_date, "%Y-%m-%d %H:%M:%S")?
                        .and_local_timezone(timezone)
                        .single()
                    {
                        Some(published_time) => Some(published_time.to_utc().into()),
                        None => None,
                    }
                }
                _ => None,
            };
            articles.push(FetchNewsArticle {
                source_name: "NewsData".to_string(),
                id: Some(result.article_id),
                link: result.link,
                title: result.title,
                short_text: result.description,
                long_text: None, // `content` field is only available in paid plans
                image_url: result.image_url,
                published_time,
            });
        }
        Ok((news_response.total_results, articles, news_response.next_page))
    }
}

#[async_trait]
impl NewsFetcher for NewsdataClient {
    async fn fetch_news(self: Arc<Self>, handler: FetchNewsHandler) -> Vec<FetchNewsOutput> {
        let mut outputs = vec![];
        let mut remaining_results_num = None;
        let mut page = None;
        let mut page_index = 0;
        loop {
            if match self.pages_num_limit {
                Some(pages_num_limit) => page_index >= pages_num_limit,
                None => match remaining_results_num {
                    Some(remaining_results_num) => remaining_results_num <= 0,
                    None => false,
                },
            } {
                break;
            }
            match self.fetch_page(page).await {
                Ok((total_results, page_articles, next_page)) => {
                    info!(
                        "page_index={}, total_results={}, page_articles.len={}, next_page={:?}",
                        page_index,
                        total_results,
                        page_articles.len(),
                        next_page,
                    );
                    remaining_results_num = Some(
                        match remaining_results_num {
                            Some(remaining_results_num) => remaining_results_num,
                            None => total_results as i32,
                        } - page_articles.len() as i32,
                    );
                    for article in page_articles {
                        outputs.push(handler(article));
                    }
                    match next_page {
                        Some(next_page) => page = Some(next_page),
                        None => break,
                    };
                    page_index += 1;
                }
                Err(error) => {
                    error!("page_index={}, error={}", page_index, error);
                    break;
                }
            }
        }
        outputs
    }
}
