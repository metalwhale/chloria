use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Local, NaiveTime};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

use crate::{
    execution::ports::repository::{InsertNewsInsightInput, Repository, SelectNewsInput, SelectNewsOutput},
    schema::{
        client_credentials::dsl::*,
        news::{self, dsl::*},
        news_insights,
    },
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

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = news_insights)]
struct InsertNewsInsightValue {
    id: i32,
    fields: String,
    updated_at: DateTime<Local>,
}

#[async_trait]
impl Repository for PostgresqlClient {
    async fn select_client_api_secret(&self, api_key_input: &str) -> Result<Option<String>> {
        let api_secret_value = client_credentials
            .filter(api_key.eq(api_key_input))
            .select(api_secret)
            .first(&mut self.pool.get()?)
            .optional()?;
        Ok(api_secret_value)
    }

    async fn select_news(&self, input: SelectNewsInput) -> Result<Vec<SelectNewsOutput>> {
        let date: DateTime<Local> = DateTime::from(input.date.and_time(NaiveTime::default()).and_utc());
        let next_date = date + Duration::days(1);
        let mut outputs = vec![];
        for (source_name_value, article_id_value, title_value, long_text_value, image_path_value) in news
            .filter(news::created_at.ge(date).and(news::created_at.lt(next_date)))
            .select((source_name, article_id, title, long_text, image_path))
            .get_results(&mut self.pool.get()?)?
        {
            outputs.push(SelectNewsOutput {
                source_name: source_name_value,
                article_id: article_id_value,
                title: title_value,
                text: long_text_value,
                image_path: image_path_value,
            });
        }
        Ok(outputs)
    }

    async fn insert_news_insight(&self, input: InsertNewsInsightInput) -> Result<()> {
        let news_id: i32 = news
            .filter(source_name.eq(input.source_name).and(article_id.eq(input.article_id)))
            .select(news::id)
            .first(&mut self.pool.get()?)?;
        let value = InsertNewsInsightValue {
            id: news_id,
            fields: input.fields,
            updated_at: Local::now(),
        };
        diesel::insert_into(news_insights::table)
            .values(&value)
            .on_conflict(news_insights::id)
            .do_update()
            .set(&value)
            .execute(&mut self.pool.get()?)?;
        Ok(())
    }
}
