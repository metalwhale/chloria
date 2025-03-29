use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::execution::ports::repository::InsertNewsInsightInput;

use super::{
    super::{ports::repository::Repository, workshop::Workshop},
    Case,
};

pub(crate) struct CreateNewsInsightCaseInput {
    pub(crate) source_name: String,
    pub(crate) article_id: String,
    pub(crate) fields: String,
}

struct CreateNewsInsightCase {
    repository: Arc<dyn Repository>,
    input: CreateNewsInsightCaseInput,
}

impl Workshop {
    pub(crate) async fn execute_create_news_insight_case(&self, input: CreateNewsInsightCaseInput) -> Result<()> {
        let case = CreateNewsInsightCase {
            repository: Arc::clone(&self.repository),
            input,
        };
        self.run_case(case).await
    }
}

#[async_trait]
impl Case for CreateNewsInsightCase {
    type Output = ();

    async fn execute(self) -> Result<Self::Output> {
        self.repository
            .insert_news_insight(InsertNewsInsightInput {
                source_name: self.input.source_name,
                article_id: self.input.article_id,
                fields: self.input.fields,
            })
            .await?;
        Ok(())
    }
}
