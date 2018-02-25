#![recursion_limit = "1024"]
#![feature(const_fn)]
#![feature(conservative_impl_trait)]
extern crate bounded_spsc_queue;
extern crate crossbeam;
#[warn(unused_imports)]
#[macro_use]
extern crate error_chain;
extern crate futures_core;
extern crate itertools;
extern crate libc;
extern crate slab;
#[macro_use]
pub extern crate slog;

#[cfg(test)]
extern crate slog_term;

extern crate smp_resource as resource;

#[cfg(test)]
#[macro_use]
pub mod test {
    use std;
    use slog::*;
    use slog_term;
    use std::sync::Arc;
}

pub mod reactor;

pub mod align;
pub mod app;
pub mod smp;
pub mod smp_message_queue;

mod sys;
