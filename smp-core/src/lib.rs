#![recursion_limit = "1024"]
#![feature(const_fn)]
#![feature(conservative_impl_trait)]
#![feature(drop_types_in_const)]
#[warn(unused_imports)]
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate error_chain;
#[macro_use]
pub extern crate slog;
extern crate scoped_tls;
extern crate nix;
extern crate libc;
extern crate tokio_core;
extern crate mio;
extern crate futures;
extern crate bounded_spsc_queue;
extern crate slab;
extern crate crossbeam;
extern crate itertools;

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

//    #[macro_export]
//    macro_rules! test {
//        (should_panic, $logger_name: ident, $name:ident, $test:block) => {
//            test!(#[should_panic], $logger_name, $name, $test);
//        };
//        ($(#[$attr:meta])*, $logger_name: ident, $name:ident, $test:block) => {
//            #[test]
//            $( #[$attr] )*
//            fn $name() {
//                let logger = test::ensure_env_logger_initialized();
//                $test
//            }
//        };
//        ($logger_name: ident, $name:ident, $test:block) => {
//            test!(, $logger_name, $name, $test);
//        };
//    }
//
//    pub fn ensure_env_logger_initialized() -> Logger {
//        let plain= slog_term::PlainSyncDecorator::new(std::io::stdout());
//        Logger::root(
//            slog_term::FullFormat::new(plain)
//                .build().fuse(), o!(),
//        )
//    }
}

pub mod reactor;

pub mod align;
pub mod app;
pub mod smp;
pub mod smp_message_queue;

mod sys;
