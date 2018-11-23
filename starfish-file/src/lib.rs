#![warn(rust_2018_idioms)]
#![feature(async_await, await_macro, futures_api)]
#![feature(tool_lints)]
#![feature(nll)]

use spdk_sys::bdev;
use spdk_sys::blob;
use spdk_sys::blob_bdev;
use std::cell::UnsafeCell;
use failure::Error;

thread_local!(static CURRENT_EXECUTOR: UnsafeCell<Option<FileSystem>> = UnsafeCell::new(None));

struct FileSystem {}

// I need to get a c

pub async fn load<S>(name: S) -> Result<(), Error>
where
    S: Into<String> + Clone,
{
    let mut bdev = bdev::get_by_name(name)?;
    println!("{:?}", bdev);

    let mut bs_dev = blob_bdev::create_bs_dev(&mut bdev)?;
    println!("{:?}", bs_dev);

    let mut blobstore = await!(blob::bs_init(&mut bs_dev))?;

    Ok(())
}
