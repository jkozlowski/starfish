use crate::blob_bdev::BlobStoreBDev;
use crate::generated;
use crate::generated::spdk_blob_bindings::{
    spdk_bs_init,
    spdk_blob_id, 
    spdk_blob_store, 
    spdk_bs_create_blob, 
    spdk_bs_get_page_size, 
    spdk_blob,
    spdk_bs_open_blob
};
use failure::Error;
use futures::channel::oneshot;
use futures::channel::oneshot::Sender;
use std::os::raw::{c_int, c_void};
use std::ptr;

#[derive(Debug, Fail)]
pub enum BlobstoreError {
    #[fail(display = "Failed to initialize blob store: {}", _0)]
    InitError(i32),
}

#[derive(Debug)]
pub struct Blobstore {
    pub(crate) blob_store: *mut spdk_blob_store,
}

impl Blobstore {
    pub fn get_page_size(&self) -> usize {
        return unsafe { spdk_bs_get_page_size(self.blob_store) } as usize;
    }
}

#[derive(Debug, Fail)]
pub enum BlobError {
    #[fail(display = "Failed to create blob: {}", _0)]
    CreateError(i32),

    #[fail(display = "Failed to open blob({}): {}", _0, _1)]
    OpenError(spdk_blob_id, i32),
}

#[derive(Debug)]
pub struct BlobId {
    pub(crate) blob_id: spdk_blob_id,
}

#[derive(Debug)]
pub struct Blob {
    pub(crate) blob: *mut spdk_blob,
}

// TODO: Implement Drop correctly with a call to spdk_bs_unload:
// Funny thing is that this is async, so will be interesting to see how to do that?
// I can't block

/// Initialize a blobstore on the given device.
pub async fn bs_init(bs_dev: &mut BlobStoreBDev) -> Result<Blobstore, Error> {
    let (sender, receiver) = oneshot::channel();

    unsafe {
        spdk_bs_init(
            // PITA that bindgen seems to generate the mappings multiple times...
            bs_dev.bs_dev as *mut generated::spdk_blob_bindings::spdk_bs_dev,
            ptr::null_mut(),
            Some(complete_callback::<*mut spdk_blob_store>),
            cb_arg(sender),
        );
    }

    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(blob_store) => return Ok(Blobstore { blob_store }),
        Err(bserrno) => {
            return Err(BlobstoreError::InitError(bserrno))?;
        }
    }
}

pub async fn create_blob(blob_store: &Blobstore) -> Result<BlobId, Error> {
    let (sender, receiver) = oneshot::channel();
    unsafe {
        spdk_bs_create_blob(
            blob_store.blob_store,
            Some(complete_callback::<spdk_blob_id>),
            cb_arg(sender),
        );
    }
    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(blob_id) => return Ok(BlobId { blob_id }),
        Err(bserrno) => {
            return Err(BlobError::CreateError(bserrno))?;
        }
    }
}

//pub async fn open_blob<'a>(blob_store: &'a Blobstore, blob_id: &'a BlobId) -> Result<Blob, Error> {
//    unimplemented!();
//
//    let (sender, receiver) = oneshot::channel();
//    unsafe {
//        spdk_bs_open_blob(
//            blob_store.blob_store,
//            blob_id.blob_id,
//            Some(complete_callback::<*mut spdk_blob>),
//            cb_arg(sender),
//        );
//    }
//    let res = await!(receiver).expect("Cancellation is not supported");
//
//    match res {
//        Ok(blob) => return Ok(Blob { blob }),
//        Err(bserrno) => {
//            return Err(BlobError::OpenError(blob_id.blob_id, bserrno))?;
//        }
//    }
//}

fn cb_arg<T>(sender: Sender<Result<T, i32>>) -> *mut c_void {
    return Box::into_raw(Box::new(sender)) as *const _ as *mut c_void;
}

extern "C" fn complete_callback<T>(sender_ptr: *mut c_void, bs: T, bserrno: c_int) {
    let sender = unsafe { Box::from_raw(sender_ptr as *mut Sender<Result<T, i32>>) };

    let ret;
    if bserrno != 0 {
        ret = Err(bserrno);
    } else {
        ret = Ok(bs);
    }

    // TODO: figure out what to do if Receiver is gone
    let _ = sender.send(ret); //.expect("Receiver is gone");
}
