use std::io;
use tokio::fs;

pub struct File {
    file: fs::File,
}

impl File {
    pub fn create(file: fs::File) -> File {
        File { file }
    }

    pub async fn truncate(&mut self, len: u64) -> io::Result<()> {
        self.file.set_len(len).await
    }
}

impl Drop for File {
    fn drop(&mut self) {}
}
