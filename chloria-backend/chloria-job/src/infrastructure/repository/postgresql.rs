use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

use crate::{
    execution::ports::repository::{InsertNewsInput, Repository},
    schema::news,
};

pub(crate) struct PostgresqlClient {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl PostgresqlClient {
    pub(crate) fn new(database_url: String) -> Result<Self> {
        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(ConnectionManager::<PgConnection>::new(database_url))?;
        Ok(Self { pool })
    }
}

#[derive(Insertable)]
#[diesel(table_name = news)]
struct InsertNewsValue {
    source_name: String,
    article_id: String,
    link: Option<String>,
    title: Option<String>,
    short_text: Option<String>,
    long_text: Option<String>,
    image_path: Option<String>,
    published_time: Option<DateTime<Local>>,
}

#[async_trait]
impl Repository for PostgresqlClient {
    async fn insert_news(&self, inputs: Vec<InsertNewsInput>) -> Result<Vec<i32>> {
        let values: Vec<InsertNewsValue> = inputs
            .into_iter()
            .map(|input| InsertNewsValue {
                source_name: input.source_name,
                article_id: input.article_id,
                link: input.link,
                title: input.title,
                short_text: input.short_text,
                long_text: input.long_text,
                image_path: input.image_path,
                published_time: input.published_time,
            })
            .collect();
        let news_ids = diesel::insert_into(news::table)
            .values(&values)
            .on_conflict_do_nothing()
            .returning(news::id)
            .get_results(&mut self.pool.get()?)?;
        Ok(news_ids)
    }
}
