#![warn(rust_2018_idioms)]
#![feature(async_await, await_macro, futures_api, extern_prelude)]

extern crate failure;
#[macro_use]
extern crate failure_derive;

extern crate futures;

mod generated;

pub mod bdev;

pub mod blob_bdev;
pub mod blob;

pub mod event;
pub mod io_channel;
pub mod nvme;
