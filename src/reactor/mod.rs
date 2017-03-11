use smp_message_queue::SmpQueues;
use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

scoped_thread_local!(static REACTOR: Reactor);

pub struct Reactor {
    id: usize,
    sleeping: Arc<AtomicBool>,
    smp_queues: SmpQueues,
}

impl fmt::Debug for Reactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reactor({})", self.id)
    }
}

impl Reactor {
    #[inline]
    pub fn with<F, R>(f: F) -> R where F: FnOnce(&Reactor) -> R {
        REACTOR.with(f)
    }

    pub fn allocate_reactor<F, R>(
        id: usize,
        sleeping: Arc<AtomicBool>,
        smp_queues: SmpQueues,
        f: F)
    where F: FnOnce(&Reactor) -> R
    {
        let reactor = Reactor {
            id: id,
            sleeping: sleeping,
            smp_queues: smp_queues
        };
        REACTOR.set(&reactor, || f(&reactor));
    }
}