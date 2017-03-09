// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]
#![feature(alloc, heap_api)]

extern crate alloc;
#[macro_use]
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate libc;
#[macro_use]
extern crate log;
extern crate nix;
#[macro_use]
extern crate tokio_core;
//extern crate tokio_proto;
extern crate tokio_smp as smp;
//extern crate tokio_service;

#[cfg(test)]
#[macro_use]
extern crate hamcrest;

//mod error;
//mod proto;
//mod service;

use alloc::heap;
use std::io;
use std::str;
use std::string::String;
use std::vec::Vec;
use nix::fcntl::O_RDONLY;

use smp::file::manager::FileManager;
use smp::file::manager::File;
use futures::Future;
use tokio_core::reactor::Core;
use tokio_core::reactor::Handle;

fn main() {
    env_logger::init().unwrap();
    match run() {
        Ok(()) => info!("OK"),
        Err(e) => error!("Err: {:?}", e)
    }
}

fn run() -> io::Result<()> {
    info!("Starting");

    let mut core: Core = Core::new()?;
    let file_manager: FileManager = FileManager::create(&core.handle())?;

    let file: File = file_manager.open_file_dma("/src/shell.sh", O_RDONLY)?;

    let read_file =
        file.read_dma(0, aligned_vec())
            .and_then(|(mut buf, read)| {
                unsafe {
                    buf.set_len(read);
                };
                let str: String = String::from_utf8(buf).unwrap();
                info!("Read \"{}\"", str);
                Ok(())
            })
            .map_err(|e| {
                error!("Cancelled: {:?}", e);
                io::Error::new(io::ErrorKind::Other, "Cancelled")
            });

    core.run(read_file)
}

fn aligned_vec() -> Vec<u8> {
    let align = 1024;
    //    let align = 4096;
    // It crashes with 4096
    let ret = unsafe { heap::allocate(align, align) };
    assert!(!ret.is_null(), "Out of memory");
    unsafe { Vec::from_raw_parts(ret, align, align) }
}
