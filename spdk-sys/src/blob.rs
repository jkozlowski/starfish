use crate::blob_bdev::BlobStoreBDev;
use crate::env::Buf;
use crate::generated::{
    spdk_blob, spdk_blob_close, spdk_blob_get_id, spdk_blob_get_num_clusters, spdk_blob_id,
    spdk_blob_io_read, spdk_blob_io_write, spdk_blob_resize, spdk_blob_store, spdk_blob_sync_md,
    spdk_bs_alloc_io_channel, spdk_bs_create_blob, spdk_bs_delete_blob, spdk_bs_free_cluster_count,
    spdk_bs_free_io_channel, spdk_bs_get_page_size, spdk_bs_init, spdk_bs_open_blob,
    spdk_bs_unload, spdk_io_channel,
};
use failure::Error;
use futures::channel::oneshot;
use futures::channel::oneshot::Sender;
use libc::c_int;
use libc::c_void;
use std::fmt;
use std::fmt::Debug;
use std::ptr;

#[derive(Debug, Fail)]
pub enum BlobstoreError {
    #[fail(display = "Failed to initialize blob store: {}", _0)]
    InitError(i32),

    #[fail(display = "Failed to allocate io channel")]
    IoChannelAllocateError,

    #[fail(display = "Failed to unload blob store: {}", _0)]
    UnloadError(i32),
}

#[derive(Debug)]
pub struct Blobstore {
    pub(crate) blob_store: *mut spdk_blob_store,
}

impl Blobstore {
    pub fn get_page_size(&self) -> u64 {
        unsafe { spdk_bs_get_page_size(self.blob_store) }
    }

    pub fn get_free_cluster_count(&self) -> u64 {
        unsafe { spdk_bs_free_cluster_count(self.blob_store) }
    }

    pub fn alloc_io_channel(&mut self) -> Result<IoChannel, Error> {
        let io_channel = unsafe { spdk_bs_alloc_io_channel(self.blob_store) };
        if io_channel.is_null() {
            return Err(BlobstoreError::IoChannelAllocateError)?;
        }
        Ok(IoChannel { io_channel })
    }
}

#[derive(Debug, Fail)]
pub enum BlobError {
    #[fail(display = "Failed to create blob: {}", _0)]
    CreateError(i32),

    #[fail(display = "Failed to open blob({}): {}", _0, _1)]
    OpenError(BlobId, i32),

    #[fail(display = "Failed to resize blob({}): {}", _0, _1)]
    ResizeError(BlobId, i32),

    #[fail(display = "Failed to sync metadata for blob({}): {}", _0, _1)]
    SyncError(BlobId, i32),

    #[fail(
        display = "Error in write completion({}): {}, offset: {}, length: {}",
        _0,
        _1,
        _2,
        _3
    )]
    WriteError(BlobId, i32, u64, u64),

    #[fail(
        display = "Error in read completion({}): {}, offset: {}, length: {}",
        _0,
        _1,
        _2,
        _3
    )]
    ReadError(BlobId, i32, u64, u64),

    #[fail(display = "Failed to close blob: {}", _0)]
    CloseError(i32),

    #[fail(display = "Failed to delete blob({}): {}", _0, _1)]
    DeleteError(BlobId, i32),
}

#[derive(Debug, Clone, Copy)]
pub struct BlobId {
    pub(crate) blob_id: spdk_blob_id,
}

impl fmt::Display for BlobId {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Blob {
    pub(crate) blob: *mut spdk_blob,
}

impl Blob {
    pub fn get_num_clusters(&self) -> u64 {
        unsafe { spdk_blob_get_num_clusters(self.blob) }
    }

    pub fn get_blob_id(&self) -> BlobId {
        let blob_id = unsafe { spdk_blob_get_id(self.blob) };
        BlobId { blob_id }
    }
}

// TODO: Drop for Blob

pub struct IoChannel {
    pub(crate) io_channel: *mut spdk_io_channel,
}

impl Drop for IoChannel {
    fn drop(&mut self) {
        unsafe { spdk_bs_free_io_channel(self.io_channel) };
    }
}

// TODO: Implement Drop correctly with a call to spdk_bs_unload:
// Funny thing is that this is async, so will be interesting to see how to do that?
// I can't block

/// Initialize a blobstore on the given device.
pub async fn bs_init(bs_dev: &mut BlobStoreBDev) -> Result<Blobstore, Error> {
    let (sender, receiver) = oneshot::channel();

    unsafe {
        spdk_bs_init(
            bs_dev.bs_dev,
            ptr::null_mut(),
            Some(complete_callback_1::<*mut spdk_blob_store>),
            cb_arg(sender),
        );
    }

    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(blob_store) => Ok(Blobstore { blob_store }),
        Err(bserrno) => Err(BlobstoreError::InitError(bserrno))?,
    }
}

pub async fn bs_unload(blob_store: Blobstore) -> Result<(), Error> {
    let (sender, receiver) = oneshot::channel();
    unsafe {
        spdk_bs_unload(
            blob_store.blob_store,
            Some(complete_callback_0),
            cb_arg::<()>(sender),
        );
    }
    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(()) => Ok(()),
        Err(bserrno) => Err(BlobstoreError::UnloadError(bserrno))?,
    }
}

pub async fn create(blob_store: &Blobstore) -> Result<BlobId, Error> {
    let (sender, receiver) = oneshot::channel();
    unsafe {
        spdk_bs_create_blob(
            blob_store.blob_store,
            Some(complete_callback_1::<spdk_blob_id>),
            cb_arg(sender),
        );
    }
    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(blob_id) => Ok(BlobId { blob_id }),
        Err(bserrno) => Err(BlobError::CreateError(bserrno))?,
    }
}

pub async fn open<'a>(blob_store: &'a Blobstore, blob_id: BlobId) -> Result<Blob, Error> {
    let (sender, receiver) = oneshot::channel();
    unsafe {
        spdk_bs_open_blob(
            blob_store.blob_store,
            blob_id.blob_id,
            Some(complete_callback_1::<*mut spdk_blob>),
            cb_arg(sender),
        );
    }
    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(blob) => Ok(Blob { blob }),
        Err(bserrno) => Err(BlobError::OpenError(blob_id, bserrno))?,
    }
}

pub async fn resize<'a>(blob: &'a Blob, required_size: u64) -> Result<(), Error> {
    let (sender, receiver) = oneshot::channel();
    unsafe {
        spdk_blob_resize(
            blob.blob,
            required_size,
            Some(complete_callback_0),
            cb_arg::<()>(sender),
        );
    }
    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(()) => Ok(()),
        Err(bserrno) => Err(BlobError::ResizeError(blob.get_blob_id(), bserrno))?,
    }
}

/**
 * Sync a blob.
 *
 * Make a blob persistent. This applies to open, resize, set xattr, and remove
 * xattr. These operations will not be persistent until the blob has been synced.
 *
 * \param blob Blob to sync.
 * \param cb_fn Called when the operation is complete.
 * \param cb_arg Argument passed to function cb_fn.
 */
/// Metadata is stored in volatile memory for performance
/// reasons and therefore needs to be synchronized with
/// non-volatile storage to make it persistent. This can be
/// done manually, as shown here, or if not it will be done
/// automatically when the blob is closed. It is always a
/// good idea to sync after making metadata changes unless
/// it has an unacceptable impact on application performance.
pub async fn sync_metadata<'a>(blob: &'a Blob) -> Result<(), Error> {
    let (sender, receiver) = oneshot::channel();
    unsafe {
        spdk_blob_sync_md(blob.blob, Some(complete_callback_0), cb_arg::<()>(sender));
    }
    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(()) => Ok(()),
        Err(bserrno) => Err(BlobError::SyncError(blob.get_blob_id(), bserrno))?,
    }
}

/// Write data to a blob.
///
/// \param blob Blob to write.
/// \param channel The I/O channel used to submit requests.
/// \param payload The specified buffer which should contain the data to be written.
/// \param offset Offset is in pages from the beginning of the blob.
/// \param length Size of data in pages.
/// \param cb_fn Called when the operation is complete.
/// \param cb_arg Argument passed to function cb_fn.
/// TODO: the interface here is funky as is, needs work;
/// Specifically writes need to happen in pages, so the buf abstraction should probably enforce that.
/// Similarly, spdk_blob_io_writev is probably the more interesting case if we don't want to
/// have to do copies.
pub async fn write<'a>(
    blob: &'a Blob,
    io_channel: &'a IoChannel,
    buf: &'a Buf,
    offset: u64,
    length: u64,
) -> Result<(), Error> {
    let (sender, receiver) = oneshot::channel();
    unsafe {
        spdk_blob_io_write(
            blob.blob,
            io_channel.io_channel,
            buf.ptr,
            offset,
            length,
            Some(complete_callback_0),
            cb_arg::<()>(sender),
        );
    }
    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(()) => Ok(()),
        Err(bserrno) => Err(BlobError::WriteError(
            blob.get_blob_id(),
            bserrno,
            offset,
            length,
        ))?,
    }
}

pub async fn read<'a>(
    blob: &'a Blob,
    io_channel: &'a IoChannel,
    buf: &'a Buf,
    offset: u64,
    length: u64,
) -> Result<(), Error> {
    let (sender, receiver) = oneshot::channel();
    unsafe {
        spdk_blob_io_read(
            blob.blob,
            io_channel.io_channel,
            buf.ptr,
            offset,
            length,
            Some(complete_callback_0),
            cb_arg::<()>(sender),
        );
    }
    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(()) => Ok(()),
        Err(bserrno) => Err(BlobError::ReadError(
            blob.get_blob_id(),
            bserrno,
            offset,
            length,
        ))?,
    }
}

pub async fn close(blob: Blob) -> Result<(), Error> {
    let (sender, receiver) = oneshot::channel();
    unsafe {
        spdk_blob_close(blob.blob, Some(complete_callback_0), cb_arg::<()>(sender));
    }
    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(()) => Ok(()),
        Err(bserrno) => Err(BlobError::CloseError(bserrno))?,
    }
}

pub async fn delete<'a>(blob_store: &'a Blobstore, blob_id: BlobId) -> Result<(), Error> {
    let (sender, receiver) = oneshot::channel();
    unsafe {
        spdk_bs_delete_blob(
            blob_store.blob_store,
            blob_id.blob_id,
            Some(complete_callback_0),
            cb_arg::<()>(sender),
        );
    }
    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(()) => Ok(()),
        Err(bserrno) => Err(BlobError::DeleteError(blob_id, bserrno))?,
    }
}

fn cb_arg<T>(sender: Sender<Result<T, i32>>) -> *mut c_void {
    Box::into_raw(Box::new(sender)) as *const _ as *mut c_void
}

extern "C" fn complete_callback_0(sender_ptr: *mut c_void, bserrno: c_int) {
    let sender = unsafe { Box::from_raw(sender_ptr as *mut Sender<Result<(), i32>>) };
    let ret = if bserrno != 0 { Err(bserrno) } else { Ok(()) };
    sender.send(ret).expect("Receiver is gone");
}

extern "C" fn complete_callback_1<T>(sender_ptr: *mut c_void, bs: T, bserrno: c_int)
where
    T: Debug,
{
    let sender = unsafe { Box::from_raw(sender_ptr as *mut Sender<Result<T, i32>>) };
    let ret = if bserrno != 0 { Err(bserrno) } else { Ok(bs) };
    sender.send(ret).expect("Receiver is gone");
}
