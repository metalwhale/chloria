use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use strum::Display;

use super::{
    super::{
        ports::{hashing_algorithm::HashingAlgorithm, repository::Repository},
        workshop::Workshop,
    },
    Case,
};

pub(crate) struct AuthenticateCaseInput {
    pub(crate) api_key: String,
    pub(crate) api_secret: String,
}

#[derive(Display)]
pub(crate) enum AuthenticateCaseOutput {
    NotFound,
    IncorrectSecret,
    Success,
}

struct AuthenticateCase {
    repository: Arc<dyn Repository>,
    hashing_algorithm: Box<dyn HashingAlgorithm>,
    input: AuthenticateCaseInput,
}

impl Workshop {
    pub(crate) async fn execute_authenticate_case(
        &self,
        input: AuthenticateCaseInput,
    ) -> Result<AuthenticateCaseOutput> {
        let case = AuthenticateCase {
            repository: Arc::clone(&self.repository),
            hashing_algorithm: self.hashing_algorithm.clone(),
            input,
        };
        self.run_case(case).await
    }
}

#[async_trait]
impl Case for AuthenticateCase {
    type Output = AuthenticateCaseOutput;

    async fn execute(self) -> Result<Self::Output> {
        let Some(api_secret_value) = self.repository.select_client_api_secret(&self.input.api_key).await? else {
            return Ok(AuthenticateCaseOutput::NotFound);
        };
        if !self
            .hashing_algorithm
            .verify(&self.input.api_secret, &api_secret_value)?
        {
            return Ok(AuthenticateCaseOutput::IncorrectSecret);
        }
        Ok(AuthenticateCaseOutput::Success)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;

    use super::{
        super::{
            super::ports::{hashing_algorithm::MockHashingAlgorithm, repository::MockRepository},
            Case,
        },
        AuthenticateCase, AuthenticateCaseInput, AuthenticateCaseOutput,
    };

    #[tokio::test]
    async fn confirm_successful_authentication() -> Result<()> {
        let mut mock_repository = MockRepository::new();
        mock_repository
            .expect_select_client_api_secret()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Some("".to_string())) }));
        let mut mock_hashing_algorithm = MockHashingAlgorithm::new();
        mock_hashing_algorithm
            .expect_verify()
            .times(1)
            .returning(|_, _| Ok(true));
        let case = AuthenticateCase {
            repository: Arc::new(mock_repository),
            hashing_algorithm: Box::new(mock_hashing_algorithm),
            input: AuthenticateCaseInput {
                api_key: "".to_string(),
                api_secret: "".to_string(),
            },
        };
        let output = case.execute().await?;
        assert!(matches!(output, AuthenticateCaseOutput::Success));
        Ok(())
    }
}
