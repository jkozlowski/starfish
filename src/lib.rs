#![recursion_limit = "1024"]
#![feature(const_fn)]
#![feature(nonzero)]
#[warn(unused_imports)]

#[macro_use] extern crate derive_builder;
#[macro_use] extern crate error_chain;
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
extern crate itertools;

#[cfg(test)]
extern crate env_logger;

#[cfg(test)]
#[macro_use]
pub mod test {

    use env_logger;
    use std::sync::{Once, ONCE_INIT};

    static LOGGER_INIT: Once = ONCE_INIT;

    #[macro_export]
    macro_rules! test {
        (should_panic, $name:ident, $test:block) => {
            test!(#[should_panic], $name, $test);
        };
        ($(#[$attr:meta])*, $name:ident, $test:block) => {
            #[test]
            $( #[$attr] )*
            fn $name() {
                test::ensure_env_logger_initialized();
                $test
            }
        };
        ($name:ident, $test:block) => {
            test!(, $name, $test);
        };
    }

    pub fn ensure_env_logger_initialized() {
        LOGGER_INIT.call_once(|| {
            env_logger::init().unwrap();
        });
    }
}

pub mod error;
pub mod eventfd;
pub mod file;
pub mod reactor;
pub mod resource;
pub mod signal;

pub mod align;
pub mod resources;
pub mod smp;
pub mod smp_message_queue;

mod sys;