use chrono::{DateTime, Local};
use rand::distr::{Alphanumeric, SampleString};

const FILE_KEY_LENGTH: usize = 16;

pub(crate) struct FileEntity {
    pub(crate) key: String,                   // Unique key to distinguish this file from others
    pub(crate) created_time: DateTime<Local>, // Date and time when the file was created
}

impl FileEntity {
    pub(crate) fn new(key: Option<String>, created_time: Option<DateTime<Local>>) -> Self {
        Self {
            key: key.unwrap_or(Alphanumeric.sample_string(&mut rand::rng(), FILE_KEY_LENGTH)),
            created_time: created_time.unwrap_or(Local::now()),
        }
    }
}
