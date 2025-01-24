use anyhow::Result;

use crate::execution::ports::file_storage::{FileObjectKind, FileStorage, UploadFileInput};

pub(crate) struct CollectNewsCase<'c> {
    file_storage: &'c dyn FileStorage,
}

impl<'c> CollectNewsCase<'c> {
    pub(crate) fn new(file_storage: &'c dyn FileStorage) -> Self {
        Self { file_storage }
    }

    pub(crate) async fn execute(&self) -> Result<()> {
        self.file_storage
            .upload_file(UploadFileInput {
                kind: FileObjectKind::Origin,
            })
            .await?;
        Ok(())
    }
}
