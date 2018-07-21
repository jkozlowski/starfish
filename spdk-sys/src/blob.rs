use failure::Error;
use generated;
use generated::spdk_blob_bindings::{
    spdk_bs_init, spdk_blob_store    
};
use blob_bdev::{ BlobStoreBDev };
use futures::channel::oneshot;
use futures::channel::oneshot::{ Sender };
use std::mem::forget;
use std::ptr;
use std::os::raw::{ c_void, c_int };

#[derive(Debug, Fail)]
pub enum BlobstoreError {
    #[fail(display = "Failed to initialize blob store: {}", _0)]
    InitError(i32),
}

#[derive(Debug)]
pub struct Blobstore {
    pub (crate) blob_store: *mut spdk_blob_store
}

// TODO: Implement Drop correctly with a call to spdk_bs_unload:
// Funny thing is that this is async, so will be interesting to see how to do that?
// I can't block

/// Initialize a blobstore on the given device.
pub async fn bs_init(bs_dev: &mut BlobStoreBDev) -> Result<Blobstore, Error> {

    extern "C" fn bs_init_complete_wrapper(
        sender_ptr: *mut c_void, bs: *mut spdk_blob_store, bserrno: c_int)
    {
        println!("Got a callback");
        let sender = unsafe {
            Box::from_raw(sender_ptr as *mut Sender<Result<*mut spdk_blob_store, i32>>)
        };
        
        let ret;
        if bserrno != 0 {
            // TODO: figure out what to do if Receiver is gone
            ret = Err(bserrno);
	    } else {
            ret = Ok(bs); 
        }
        sender.send(ret).expect("Receiver is gone");
    }

    println!("Here");

    let (sender, receiver) = oneshot::channel::<Result<*mut spdk_blob_store, i32>>();
    let sender_pointer = Box::into_raw(Box::new(sender)) as *const _ as *mut c_void;

    unsafe {
        spdk_bs_init(
            // PITA that bindgen seems to generate the mappings multiple times...
            bs_dev.bs_dev as *mut generated::spdk_blob_bindings::spdk_bs_dev, 
            ptr::null_mut(),
            Some(bs_init_complete_wrapper),
            sender_pointer
        );
    }

    let res = await!(receiver).expect("Cancellation is not supported");

    match res {
        Ok(blob_store) => { 
            return Ok(Blobstore { blob_store })
        }
        Err(bserrno) => {
            return Err(BlobstoreError::InitError(bserrno))?;
        }
    }
}

// TODO: Unload BS
// static void
// unload_bs(struct hello_context_t *hello_context, char *msg, int bserrno)
// {
// 	if (bserrno) {
// 		SPDK_ERRLOG("%s (err %d)\n", msg, bserrno);
// 		hello_context->rc = bserrno;
// 	}
// 	if (hello_context->bs) {
// 		if (hello_context->channel) {
// 			spdk_bs_free_io_channel(hello_context->channel);
// 		}
// 		spdk_bs_unload(hello_context->bs, unload_complete, hello_context);
// 	} else {
// 		spdk_app_stop(bserrno);
// 	}
// }