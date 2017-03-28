use slog::Logger;
use smp::Smp;

pub fn run<F>(log: Logger, f: F)
    where F: FnOnce()
{
    Smp::configure(log);
}

#[cfg(test)]
mod tests {
    use super::*;
    use smp::*;
    use slog_scope;
    use test;

    test!(it_works, {
        run(slog_scope::logger(), || {});
    });
}
