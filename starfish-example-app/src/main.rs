#![feature(async_await, await_macro, futures_api)]
#[macro_use]
extern crate futures;

extern crate spdk_sys as spdk;
extern crate starfish_executor as executor;

use futures::future;
use std::env;
use std::mem;
use spdk::io_channel;
use spdk::event::AppOpts;
use spdk::bdev;
use spdk::blob_bdev;
use spdk::blob;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let config_file = &args[1];
    let mut opts = AppOpts::new();

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
}

async fn run() {
    
    let mut bdev = bdev::get_by_name("Malloc0").expect("bdev not found");
    println!("{:?}", bdev);

    let mut bs_dev = blob_bdev::create_bs_dev(&mut bdev).expect("could not create bs_dev");
    println!("{:?}", bs_dev);

    let ret = await!(blob::bs_init(&mut bs_dev));
    println!("Initted! {:?}", ret);
}
