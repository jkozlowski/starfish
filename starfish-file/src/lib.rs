#![warn(rust_2018_idioms)]
#![allow(macro_use_extern_crate)]
#[macro_use]
extern crate err_derive;

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate slog;

pub type Lsn = u64;
pub type BlockNum = i64;
pub type FileId = u64;

pub mod commitlog;
pub mod fs;
mod future;

mod shared;

pub use shared::Shared;

mod executor;

pub use executor::spawn;

pub const fn align_up(len: usize, align: usize) -> usize {
    (len + align - 1) & !(align - 1)
}
