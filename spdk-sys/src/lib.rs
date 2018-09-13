#![warn(rust_2018_idioms)]
#![feature(
    async_await,
    await_macro,
    futures_api
)]
#![allow(macro_use_extern_crate)]
#[macro_use]
extern crate failure_derive;

mod generated;

pub mod bdev;

pub mod blob;
pub mod blob_bdev;

pub mod env;
pub mod event;
pub mod io_channel;
pub mod nvme;
