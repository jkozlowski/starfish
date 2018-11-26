#![warn(rust_2018_idioms)]
#![feature(async_await, await_macro, futures_api)]
#![feature(nll)]
#![allow(macro_use_extern_crate)]
#[macro_use]
extern crate failure_derive;

mod generated;

pub mod bdev;

pub mod blob;
pub mod blob_bdev;

pub mod env;
pub mod event;
pub mod io_channel;

#[cfg(test)]
#[macro_use]
extern crate hamcrest2;

#[cfg(test)]
mod ete_test {

    use crate::bdev;
    use crate::blob;
    use crate::blob_bdev;
    use crate::env as spdk_env;
    use crate::event;
    use crate::io_channel;
    use crate::io_channel::PollerHandle;
    use failure::Error;
    use hamcrest2::prelude::*;
    use starfish_executor as executor;
    use std::mem;
    use std::path::Path;

    #[test]
    pub fn ete_test() {
        let config_file = Path::new("config/hello_blob.conf").canonicalize().unwrap();
        let mut opts = event::AppOpts::new();

        opts.name("hello_blob");
        opts.config_file(config_file.to_str().unwrap());

        let ret = opts.start(|| {
            let executor = executor::initialize();

            // TODO: fixup
            mem::forget(executor);

            // Register the executor poller
            let poller = io_channel::poller_register(executor::pure_poll);

            executor::spawn(run(poller));
        });

        assert_that!(ret, is(ok()));
    }

    async fn run(poller: PollerHandle) {
        match await!(run_inner()) {
            Ok(_) => println!("Successful"),
            Err(err) => println!("Failure: {:?}", err),
        }

        drop(poller);

        event::app_stop(true);
    }

    async fn run_inner() -> Result<(), Error> {
        let mut bdev = bdev::get_by_name("AIO1")?;
        println!("{:?}", bdev);

        let mut bs_dev = blob_bdev::create_bs_dev(&mut bdev)?;
        println!("{:?}", bs_dev);

        let mut blobstore = await!(blob::bs_init(&mut bs_dev))?;

        await!(run_with_blob_store(&mut blobstore))?;

        await!(blob::bs_unload(blobstore))?;

        Ok(())
    }

    async fn run_with_blob_store(blobstore: &mut blob::Blobstore) -> Result<(), Error> {
        let page_size = blobstore.get_page_size();

        println!("Page size: {:?}", page_size);

        let blob_id = await!(blob::create(&blobstore))?;

        println!("Blob created: {:?}", blob_id);

        let blob = await!(blob::open(&blobstore, blob_id))?;

        println!("Opened blob");

        let free_clusters = blobstore.get_free_cluster_count();
        println!("blobstore has FREE clusters of {:?}", free_clusters);

        await!(blob::resize(&blob, free_clusters))?;

        let total = blob.get_num_clusters();
        println!("resized blob now has USED clusters of {}", total);

        await!(blob::sync_metadata(&blob))?;

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
        await!(blob::close(blob))?;
        println!("Closed");

        await!(blob::delete(&blobstore, blob_id))?;

        println!("Deleted");

        Ok(())
    }
}
