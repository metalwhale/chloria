use std::{io::Cursor, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;
use csv::Writer;
use log::error;
use tokio_util::io::ReaderStream;

use super::{
    super::{
        ports::repository::{Repository, SelectNewsInput},
        workshop::Workshop,
    },
    Case,
};

pub(crate) struct GetNewsCaseInput {
    pub(crate) date: NaiveDate,
}

pub(crate) struct GetNewsCaseOutput {
    pub(crate) articles_stream: ReaderStream<Cursor<Vec<u8>>>,
}

struct GetNewsCase {
    repository: Arc<dyn Repository>,
    input: GetNewsCaseInput,
}

impl Workshop {
    pub(crate) async fn execute_get_news_case(&self, input: GetNewsCaseInput) -> Result<GetNewsCaseOutput> {
        let case = GetNewsCase {
            repository: Arc::clone(&self.repository),
            input,
        };
        self.run_case(case).await
    }
}

#[async_trait]
impl Case for GetNewsCase {
    type Output = GetNewsCaseOutput;

    async fn execute(self) -> Result<Self::Output> {
        let mut writer = Writer::from_writer(vec![]);
        for select_news_output in self
            .repository
            .select_news(SelectNewsInput { date: self.input.date })
            .await?
        {
            if let Err(error) = writer.serialize(select_news_output) {
                error!("error={}", error);
            }
        }
        let output = GetNewsCaseOutput {
            articles_stream: ReaderStream::new(Cursor::new(writer.into_inner()?)),
        };
        Ok(output)
    }
}
