use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use chrono_tz::Tz;
use log::info;
use reqwest::Client;
use serde::Deserialize;
use tokio::task::JoinHandle;

use crate::execution::ports::news_fetcher::{FetchNewsOutput, NewsFetcher};

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

    async fn fetch_page(&self, page: Option<String>) -> Result<(u16, Vec<FetchNewsOutput>, Option<String>)> {
        // Doc: https://newsdata.io/news-sources/Japan-news-api
        // let mut url = format!("https://newsdata.io/api/1/latest?country=jp&apikey={}", self.api_key);
        // if let Some(page) = page {
        //     url.push_str(&format!("&page={page}"));
        // }
        // let response_text = Client::new().get(url).send().await?.text().await?;
        // let news_response: NewsResponse = serde_json::from_str(&response_text)?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let news_response = NewsResponse {
            total_results: 10,
            results: vec![
                NewsResponseResult {
                    article_id: "1".to_string(),
                    link: None,
                    title: None,
                    description: None,
                    image_url: Some("1".to_string()),
                    pub_date: None,
                    pub_date_tz: None,
                },
                NewsResponseResult {
                    article_id: "2".to_string(),
                    link: None,
                    title: None,
                    description: None,
                    image_url: Some("2".to_string()),
                    pub_date: None,
                    pub_date_tz: None,
                },
                NewsResponseResult {
                    article_id: "3".to_string(),
                    link: None,
                    title: None,
                    description: None,
                    image_url: Some("3".to_string()),
                    pub_date: None,
                    pub_date_tz: None,
                },
            ],
            next_page: Some("".to_string()),
        };
        let mut output = vec![];
        for result in news_response.results.into_iter() {
            let published_time = if let (Some(pub_date), Some(pub_date_tz)) = (result.pub_date, result.pub_date_tz) {
                match pub_date_tz.parse::<Tz>()? {
                    timezone => {
                        if let Some(published_time) = NaiveDateTime::parse_from_str(&pub_date, "%Y-%m-%d %H:%M:%S")?
                            .and_local_timezone(timezone)
                            .single()
                        {
                            Some(published_time.to_utc().into())
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            };
            output.push(FetchNewsOutput {
                source_code: "NewsData".to_string(),
                id: Some(result.article_id),
                link: result.link,
                title: result.title,
                short_text: result.description,
                long_text: None, // `content` field is only available in paid plans
                image_url: result.image_url,
                published_time,
            });
        }
        Ok((news_response.total_results, output, news_response.next_page))
    }
}

#[async_trait]
impl NewsFetcher for NewsdataClient {
    async fn fetch_news(
        &self,
        handler: Box<dyn Fn(FetchNewsOutput) -> JoinHandle<Result<()>> + Send>,
    ) -> Vec<JoinHandle<Result<()>>> {
        let mut output = vec![];
        let mut remaining_results_num = None;
        let mut page = None;
        let mut page_count = 0;
        loop {
            if match self.pages_num_limit {
                Some(pages_num_limit) => page_count >= pages_num_limit,
                None => match remaining_results_num {
                    Some(remaining_results_num) => remaining_results_num <= 0,
                    None => false,
                },
            } {
                break;
            }
            if let Ok((total_results, page_output, next_page)) = self.fetch_page(page).await {
                info!(
                    "page_count={}, page_output.len={}, total_results={}, next_page={:?}",
                    page_count,
                    page_output.len(),
                    total_results,
                    next_page,
                );
                remaining_results_num = Some(
                    match remaining_results_num {
                        Some(remaining_results_num) => remaining_results_num,
                        None => total_results,
                    } - page_output.len() as u16,
                );
                for news in page_output {
                    output.push(handler(news));
                }
                match next_page {
                    Some(next_page) => page = Some(next_page),
                    None => break,
                };
                page_count += 1;
            } else {
                break;
            }
        }
        output
    }
}
