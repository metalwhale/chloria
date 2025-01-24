use crate::execution::ports::file_storage::FileStorage;

pub(crate) struct Config<'c> {
    pub(crate) file_storage: &'c dyn FileStorage,
}

impl<'c> Config<'c> {
    pub(crate) fn new(file_storage: &'c dyn FileStorage) -> Self {
        Self { file_storage }
    }
}
