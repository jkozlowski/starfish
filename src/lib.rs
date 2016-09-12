#![recursion_limit = "1024"]
#[warn(unused_imports)]

#[macro_use] extern crate error_chain;
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;
extern crate nix;
extern crate libc;

pub mod resource;
pub mod resources;
pub mod align;
pub mod error;
pub mod smp;
