#![recursion_limit = "1024"]
#[warn(unused_imports)]

extern crate hwloc;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;


pub mod error;
pub mod smp;
pub mod resources;
