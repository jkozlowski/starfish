#![warn(rust_2018_idioms)]
#![feature(async_await, await_macro, futures_api)]
#![feature(tool_lints)]
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
    use crate::blob::Blob;
    use crate::blob::BlobId;
    use crate::blob::Blobstore;
    use crate::blob::IoChannel;
    use crate::blob_bdev;
    use crate::env as spdk_env;
    use crate::event;
    use crate::io_channel;
    use crate::io_channel::PollerHandle;
    use failure::Error;
    use futures::Future;
    use hamcrest2::prelude::*;
    use starfish_executor as executor;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::mem;
    use std::path::Path;
    use std::process::Command;
    use tempfile;
    use nix;
    use nix::sys::wait::WaitStatus;

    macro_rules! ete_test {
        ($test_name: ident, $fn_body:expr) => {
            #[test]
            fn $test_name() -> Result<(), Error> {
                let tmp_dir = tempfile::tempdir()?;

                let config_file = tmp_dir.path().join("config.conf");
                let aio_file = tmp_dir.path().join("aiofile");

                {
                    let file = File::create(&aio_file)?;
                    drop(file);

                    let tmp_aio_file_arg = format!("of={}", &aio_file.to_str().unwrap());
                    Command::new("dd")
                        .args(&["if=/dev/zero", &tmp_aio_file_arg, "bs=2048", "count=5000"])
                        .output()?;
                }

                {
                    let mut buffer = File::create(&config_file)?;
                    write!(
                        buffer,
                        "[AIO]
                        # AIO <file name> <bdev name> [<block size>]
                        AIO {:?} AIO1 2048",
                        aio_file.canonicalize().unwrap()
                    ).expect("Failed to write config file");
                }

                match nix::fork() {
                    Ok(ForkResult::Parent { child, .. }) => {
                        let res = nix::sys::wait();
                        assert_that!(res, is(ok()));

                        match res.unwrap() {
                            WaitStatus(_, status_code) => 
                        }
                    },
                    Ok(ForkResult::Child) => {
                        let mut opts = event::AppOpts::new();

                        opts.name("hello_blob");
                        opts.config_file(config_file.canonicalize().unwrap().to_str().unwrap());

                        let ret = opts.start(|| {
                            let executor = executor::initialize();

                            // TODO: fixup
                            mem::forget(executor);

                            // Register the executor poller
                            let poller = io_channel::poller_register(executor::pure_poll);

                            executor::spawn(
                                async {
                                    let res: Result<(), Error> = await!(
                                        async {
                                            let mut bdev = bdev::get_by_name("AIO1")?;
                                            let mut bs_dev = blob_bdev::create_bs_dev(&mut bdev)?;
                                            let mut blobstore = await!(blob::bs_init(&mut bs_dev))?;
                                            let channel = blobstore.alloc_io_channel()?;

                                            await!($fn_body(&mut blobstore, &channel))?;

                                            // Needs to be dropped manually,
                                            // since blobstore does reference counting for channels
                                            // and will refuse to unload if there are some still open.
                                            mem::drop(channel);

                                            await!(blob::bs_unload(blobstore))?;

                                            Ok(())
                                        }
                                    );
                                    drop(poller);

                                    event::app_stop(res.is_ok());
                                },
                            );
                        });

                        assert_that!(ret, is(ok()));

                        Ok(())
                    },
                    Err(_) => abort!("Fork failed")
                }
            }
        };
    }

    ete_test!(
        test_create_write_delete_single_blob,
        create_write_delete_single_blob
    );

    ete_test!(
        test_create_write_delete_single_blob1,
        create_write_delete_single_blob
    );

    async fn create_write_delete_single_blob<'a>(
        blobstore: &'a mut Blobstore,
        channel: &'a IoChannel,
    ) -> Result<(), Error> {
        let blob_id = await!(create_write_read_single_page_blob(
            blobstore, &channel, 0x5a
        ))?;

        await!(blob::delete(&blobstore, blob_id))?;

        Ok(())
    }

    async fn create_write_read_single_page_blob<'a>(
        blobstore: &'a mut blob::Blobstore,
        channel: &'a IoChannel,
        val: i8,
    ) -> Result<BlobId, Error> {
        let blob_id = await!(blob::create(&blobstore))?;
        let blob = await!(blob::open(&blobstore, blob_id))?;

        await!(blob::resize(&blob, 1));

        let total = blob.get_num_clusters();
        await!(blob::sync_metadata(&blob));

        assert_that!(total, is(equal_to(1)));

        let page_size = blobstore.get_page_size();
        await!(write_read_single_page_blob(&channel, &blob, page_size, val))?;

        await!(blob::close(blob))?;

        Ok(blob_id)
    }

    async fn write_read_single_page_blob<'a>(
        channel: &'a IoChannel,
        blob: &'a Blob,
        page_size: u64,
        val: i8,
    ) -> Result<(), Error> {
        let mut write_buf = spdk_env::dma_malloc(page_size, 0x1000);
        write_buf.fill(val);

        await!(blob::write(&blob, &channel, &write_buf, 0, 1))?;

        let read_buf = spdk_env::dma_malloc(page_size, 0x1000);
        await!(blob::read(&blob, &channel, &read_buf, 0, 1))?;

        assert_that!(write_buf, is(equal_to(read_buf)));
        Ok(())
    }
}
