#![recursion_limit = "1024"]
#![feature(const_fn)]
#![feature(nonzero)]
#[warn(unused_imports)]

#[macro_use] extern crate error_chain;
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;
#[macro_use] extern crate log;
#[macro_use] extern crate scoped_tls;
extern crate nix;
extern crate libc;
extern crate tokio_core;
extern crate mio;
extern crate futures;
extern crate bounded_spsc_queue;
extern crate state;
extern crate thread_scoped;
extern crate slab;
extern crate crossbeam;
extern crate core;
extern crate itertools;
#[cfg(test)] extern crate env_logger;

pub mod resource;
pub mod resources;
pub mod align;
pub mod error;
pub mod smp;
pub mod smp_message_queue;
pub mod file;
pub mod eventfd;
pub mod signal;
pub mod reactor;