#![warn(rust_2018_idioms)]
#![feature(async_await, await_macro, futures_api, nll)]

use failure::Error;
use spdk_sys::bdev;
use spdk_sys::blob;
use spdk_sys::blob_bdev;
use spdk_sys::env as spdk_env;
use spdk_sys::event;
use spdk_sys::io_channel;
use starfish_executor as executor;
use std::env;
use std::mem;

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
        io_channel::poller_register(executor::pure_poll);

        executor::spawn(run());
    });

    println!("Finished: {:?}", ret);
}

async fn run() {
    match await!(run_inner()) {
        Ok(_) => println!("Successful"),
        Err(err) => println!("Failure: {:?}", err),
    }
}

async fn run_inner() -> Result<(), Error> {
    let mut bdev = bdev::get_by_name("Malloc0")?;
    println!("{:?}", bdev);

    let mut bs_dev = blob_bdev::create_bs_dev(&mut bdev)?;
    println!("{:?}", bs_dev);

    let mut blobstore = await!(blob::bs_init(&mut bs_dev))?;
    let page_size = blobstore.get_page_size();

    println!("Page size: {:?}", page_size);

    let blob_id = await!(blob::create(&blobstore))?;

    println!("Blob created: {:?}", blob_id);

    let blob = await!(blob::open(&blobstore, &blob_id))?;

    println!("Opened blob");

    let free_clusters = blobstore.get_free_cluster_count();
    println!("blobstore has FREE clusters of {:?}", free_clusters);

    await!(blob::resize(&blob, free_clusters));

    let total = blob.get_num_clusters();
    println!("resized blob now has USED clusters of {}", total);

    await!(blob::sync_metadata(&blob));

    println!("metadata sync complete");

    /*
     * Buffers for data transfer need to be allocated via SPDK. We will
     * tranfer 1 page of 4K aligned data at offset 0 in the blob.
     */
    let mut write_buf = spdk_env::dma_malloc(page_size, 0x1000);
    write_buf.fill(0x5a);

    /* Now we have to allocate a channel. */
    let channel = blobstore.alloc_io_channel()?;

    /* Let's perform the write, 1 page at offset 0. */
    println!("Starting write");
    await!(blob::write(&blob, &channel, &write_buf, 0, 1))?;
    println!("Finished writing");

    let read_buf = spdk_env::dma_malloc(page_size, 0x1000);

    /* Issue the read */
    println!("Starting read");
    await!(blob::read(&blob, &channel, &read_buf, 0, 1))?;
    println!("Finished read");

    /* Now let's make sure things match. */
    if write_buf != read_buf {
        println!("Error in data compare");
    // unload_bs(hello_context, "Error in data compare", -1);
    // return;
    } else {
        println!("read SUCCESS and data matches!");
    }

    /* Now let's close it and delete the blob in the callback. */
    //spdk_blob_close(hello_context->blob, delete_blob, hello_context);

    event::app_stop(true);

    Ok(())
}
