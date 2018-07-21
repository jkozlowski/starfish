#![feature(async_await, await_macro, futures_api)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;

#[macro_use]
extern crate futures;

mod generated;

pub mod bdev;

pub mod blob_bdev;
pub mod blob;

pub mod event;
pub mod io_channel;
pub mod nvme;
