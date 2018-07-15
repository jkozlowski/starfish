extern crate spdk_sys as spdk;

use std::env;
use spdk::event::AppOpts;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let config_file = &args[1];
    let mut opts = AppOpts::new();

    opts.name("hello_blob");
    opts.config_file(config_file.as_str());

    let ret = opts.start(|| {
        println!("Running");
    });
}
