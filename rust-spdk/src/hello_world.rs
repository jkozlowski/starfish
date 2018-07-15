extern crate rust_spdk as spdk;

use spdk::event::AppOpts;

pub fn main() {
    let mut opts = AppOpts::new();

    opts.name("hello_blob");
    opts.config_file("/home/ec2-user/code/starfish/rust-spdk/config/hello_blob.conf");

    let ret = opts.start(|| {
        println!("Running");
    });
}
