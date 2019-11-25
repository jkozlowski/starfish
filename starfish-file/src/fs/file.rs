use std::io;
use std::io::ErrorKind::Other;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;
use std::task::Poll;
use std::task::Poll::Pending;
use std::task::Poll::Ready;

use bytes::Bytes;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::Shared;

#[derive(Clone)]
pub struct File {
    inner: Shared<Inner>,
}

struct Inner {
    file: fs::File,
    path: PathBuf,
    open_options: OpenOptions,
}

impl File {
    pub fn create(file: fs::File, path: PathBuf, open_options: OpenOptions) -> File {
        File {
            inner: Shared::new(Inner {
                file,
                path,
                open_options,
            }),
        }
    }

    pub async fn truncate(&mut self, len: u64) -> io::Result<()> {
        let mut inner = self.inner.borrow_mut();
        inner.file.set_len(len).await
    }

    pub async fn write<F>(&self, pos: SeekFrom, buf: Bytes, finalizer: F) -> io::Result<()>
    where
        F: Fn(Bytes) -> (),
    {
        let res = File::open_seek_write(
            self.inner.open_options.clone(),
            self.inner.path.clone(),
            pos,
            &buf,
        )
        .await;
        finalizer(buf);
        res
    }

    pub async fn flush(&self) -> io::Result<()> {
        unimplemented!()
    }

    async fn reopen<P>(open_options: OpenOptions, path: P) -> io::Result<fs::File>
    where
        P: AsRef<Path> + Send + Unpin + 'static,
    {
        open_options.open(path).await
    }

    async fn open_seek_write<P>(
        open_options: OpenOptions,
        path: P,
        pos: SeekFrom,
        buf: &[u8],
    ) -> io::Result<()>
    where
        P: AsRef<Path> + Send + Unpin + 'static,
    {
        // TODO(jakubk): I think this could be optimised to keep the one File if the
        // code is not submitting writes/reads fast enough.
        // So could try to get mutable access and if that fails, clone.
        let mut tokio_file = open_options.open(path).await?;
        tokio_file.seek(pos).await?;
        tokio_file.write_all(buf).await?;
        // Unfortunately this is necessary
        tokio_file.flush().await?;
        tokio_file.shutdown().await
    }
}

impl Drop for File {
    fn drop(&mut self) {}
}
