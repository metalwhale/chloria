use anyhow::Result;
use async_trait::async_trait;

pub(crate) enum FileObjectKind {
    Origin,
}

pub(crate) struct UploadFileInput {
    pub(crate) kind: FileObjectKind,
}

#[async_trait]
pub(crate) trait FileStorage {
    async fn upload_file(&self, input: UploadFileInput) -> Result<()>;
}
