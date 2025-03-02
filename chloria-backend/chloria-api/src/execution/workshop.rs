use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Semaphore;

use super::{
    cases::Case,
    ports::{hashing_algorithm::HashingAlgorithm, repository::Repository},
};

pub(crate) struct Config {
    pub(crate) case_permits_num: usize,
}

#[derive(Clone)]
pub(crate) struct Workshop {
    pub(super) repository: Arc<dyn Repository>,
    // NOTE: We could use `Arc`, but the hashing algorithm doesn't need to be shared across threads since it should have no state.
    // We use `Box` for simple cloning and to avoid overhead.
    pub(super) hashing_algorithm: Box<dyn HashingAlgorithm>,
    semaphore: Arc<Semaphore>,
}

impl Workshop {
    pub(crate) fn new(
        repository: Arc<dyn Repository>,
        hashing_algorithm: Box<dyn HashingAlgorithm>,
        config: Config,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.case_permits_num));
        Self {
            repository,
            hashing_algorithm,
            semaphore,
        }
    }

    pub(super) async fn run_case<C: Case>(&self, case: C) -> Result<C::Output> {
        let semaphore = Arc::clone(&self.semaphore);
        tokio::task::spawn(async move {
            let _permit = semaphore.acquire().await?;
            case.execute().await
        })
        .await?
    }
}
