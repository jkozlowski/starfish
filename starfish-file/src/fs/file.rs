use std::io;
use tokio::fs;
use std::task::Poll::Ready;
use std::task::Poll::Pending;
use std::task::Poll;
use std::io::ErrorKind::Other;
use std::path::PathBuf;
use std::path::Path;
use tokio::fs::OpenOptions;
use std::io::SeekFrom;
use bytes::Bytes;
use tokio::io::AsyncWriteExt;

pub struct File {
    file: fs::File,
    path: PathBuf,
    open_options: OpenOptions,
}

impl File {
    pub fn create(file: fs::File, path: PathBuf, open_options: OpenOptions) -> File {
        File { file, path, open_options }
    }

    pub async fn truncate(&mut self, len: u64) -> io::Result<()> {
        self.file.set_len(len).await
    }

    pub async fn write<F>(&mut self, pos: SeekFrom, buf: Bytes, finalizer: F) -> io::Result<()>
        where F: Fn(Bytes) -> () {
        let res = File::open_seek_write(
            self.open_options.clone(),
            self.path.clone(),
            pos,
            &buf).await;
        finalizer(buf);
        res
    }

    async fn open_seek_write<P>(open_options: OpenOptions,
                                path: P,
                                pos: SeekFrom,
                                buf: &[u8]) -> io::Result<()>
        where P: AsRef<Path> + Send + Unpin + 'static {
        let mut tokio_file = open_options.open(path).await?;
        tokio_file.seek(pos).await?;
        tokio_file.write_all(buf).await
    }
}

impl Drop for File {
    fn drop(&mut self) {}
}
