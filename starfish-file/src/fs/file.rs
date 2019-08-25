use std::io;
use tokio::fs;

pub struct File {
    file: fs::File,
}

impl File {
    pub fn create(file: fs::File) -> File {
        File { file }
    }

    pub async fn truncate(&self, length: u64) -> io::Result<()> {
        unimplemented!();
    }
}

impl Drop for File {
    fn drop(&mut self) {}
}
