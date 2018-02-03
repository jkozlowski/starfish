extern crate gcc;

use gcc::Config;
use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

fn main() {
    if cfg!(target_os = "macos") {
        let mut config: Config = gcc::Config::new();
        config.file("src/mac.c");
        config.compile("libmac.a");
    }

    return;
}

