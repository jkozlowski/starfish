#![recursion_limit = "1024"]
#![feature(const_fn)]
#![feature(nonzero)]
#[warn(unused_imports)]

#[macro_use] extern crate derive_builder;
#[macro_use] extern crate error_chain;
#[macro_use] pub extern crate slog;
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
extern crate slog_term;
#[cfg(test)]
extern crate slog_scope;

#[cfg(test)]
#[macro_use]
pub mod test {
    use std;
    use slog::*;
    use slog_term;
    use slog_scope;
    use std::sync::Arc;
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
        let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
        let root = Logger::root(
            Arc::new(slog_term::FullFormat::new(plain).build().fuse()),
            o!("version" => env!("CARGO_PKG_VERSION"))
        );
        slog_scope::set_global_logger(root.to_erased());
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