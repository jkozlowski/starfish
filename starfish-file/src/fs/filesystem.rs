use crate::fs::File;
use std::fs::OpenOptions as StdOpenOptions;
use std::io;
use std::path::Path;
use tokio::fs::OpenOptions as TokioOpenOptions;
use tokio_fs::read_dir;
use tokio_fs::ReadDir;

#[derive(Clone)]
pub struct FileSystem {}

impl FileSystem {
    pub async fn create() -> io::Result<FileSystem> {
        Ok(FileSystem {})
    }

    pub async fn open<P>(&self, path: P, options: StdOpenOptions) -> io::Result<File>
    where
        // TODO(jkozlowski): Get rid of this limitation
        P: AsRef<Path> + Send + Unpin + 'static,
    {
        let tokio_options = TokioOpenOptions::from(options);
        let file = tokio_options.open(path).await?;
        Ok(File::create(file))
    }

    pub async fn read_dir<P>(&self, path: P) -> io::Result<ReadDir>
    where
        P: AsRef<Path> + Send + 'static,
    {
        read_dir(path).await
    }
}
