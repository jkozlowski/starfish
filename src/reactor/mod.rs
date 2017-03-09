use smp_message_queue::SmpQueues;

scoped_thread_local!(static REACTOR: Reactor);
scoped_thread_local!(static SMP_QUEUES: SmpQueues);

#[derive(Debug)]
pub struct Reactor {
    id: usize,
}

impl Reactor {
    #[inline]
    pub fn with<F, R>(f: F) -> R where F: FnOnce(&Reactor) -> R {
        REACTOR.with(f)
    }

    pub fn allocate_reactor<F, R>(id: usize, f: F) where F: FnOnce(&Reactor) -> R {
        let reactor = Reactor {
            id: id,
            //            sleeping: AtomicBool::new(false)
        };
        REACTOR.set(&reactor, || f(&reactor));
    }

    pub fn assign_queues<F, R>(smp_queue: &SmpQueues, f: F) where F: FnOnce() -> R {
        SMP_QUEUES.set(&smp_queue, f);
    }
}