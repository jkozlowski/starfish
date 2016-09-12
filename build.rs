extern crate gcc;

use gcc::{Config};

fn main() {

    let mut config: Config = gcc::Config::new();
    config.file("src/resource/mac.c");
    config.compile("libmac.a");

    return
}