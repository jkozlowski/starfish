#![warn(rust_2018_idioms)]
#![feature(async_await, await_macro, futures_api)]
#[macro_use]
extern crate futures;

extern crate failure;
extern crate spdk_sys as spdk;
extern crate starfish_executor as executor;

use failure::Error;
use futures::future;
use std::env;
use std::mem;
use spdk::io_channel;
use spdk::event;
use spdk::bdev;
use spdk::blob_bdev;
use spdk::blob;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let config_file = &args[1];
    let mut opts = event::AppOpts::new();

    opts.name("hello_blob");
    opts.config_file(config_file.as_str());

    let ret = opts.start(|| {
        let executor = executor::initialize();
        
        // TODO: fixup
        mem::forget(executor);

        // Register the executor poller
        io_channel::poller_register(|| {
            return executor::pure_poll();
        });

        executor::spawn(run());
    });

    println!("Finished: {:?}", ret);
}

async fn run() {
    match await!(run_inner()) {
        Ok(_) => println!("Successful"),
        Err(err) => println!("Failure: {:?}", err)
    }
}

async fn run_inner() -> Result<(), Error> {
    
    let mut bdev = bdev::get_by_name("Malloc0")?;
    println!("{:?}", bdev);

    let mut bs_dev = blob_bdev::create_bs_dev(&mut bdev)?;
    println!("{:?}", bs_dev);

    let blobstore = await!(blob::bs_init(&mut bs_dev))?;
    let page_size = blobstore.get_page_size();

    println!("Page size: {:?}", page_size);

    let blob = await!(blob::create_blob(&blobstore))?;

    println!("Blob created: {:?}", blob);a
    
    event::app_stop(true);

    return Ok(());
}
