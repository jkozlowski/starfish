use slog::Logger;
use smp::Smp;

pub fn run<F>(log: Logger, f: F)
    where F: FnOnce()
{
    Smp::configure(log);
}

#[cfg(test)]
#[macro_use]
mod tests {
    use super::*;
    use smp::*;
    use test;
    use slog::*;
    use slog_term;
    use std;

    #[test]
    fn it_works() {
        let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
        let log = Logger::root(
            slog_term::FullFormat::new(plain)
                .build().fuse(), o!(),
        );
        run(log, || {});
    }
//    test!(logger, it_works, {
//        run(logger, || {});
//    });
}
