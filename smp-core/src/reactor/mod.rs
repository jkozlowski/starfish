use slog::Logger;
use smp_message_queue::SmpPollFn;
use smp_message_queue::SmpQueues;
use std::cell::RefCell;
use std::cell::UnsafeCell;
use std::fmt;
use std::mem;
use std::ptr::null;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

thread_local! {
    static REACTOR: UnsafeCell<*const Reactor> = UnsafeCell::new(null());
}

pub struct Reactor {
    id: usize,
    pollers: RefCell<Vec<Box<PollFn>>>,
    sleeping: Arc<AtomicBool>,
    log: Logger,
    smp_queues: SmpQueues,
}

impl fmt::Debug for Reactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reactor({})", self.id)
    }
}

#[inline]
pub fn local() -> &'static Reactor {
    REACTOR.with(|l| unsafe { mem::transmute(*l.get()) })
}

pub fn create_reactor(
    id: usize,
    log: Logger,
    sleeping: Arc<AtomicBool>,
    smp_queues: SmpQueues,
) -> &'static mut Reactor {
    let reactor = Reactor {
        id: id,
        pollers: RefCell::new(Vec::new()),
        log: log,
        sleeping: sleeping,
        smp_queues: smp_queues,
    };

    REACTOR.with(|l| unsafe {
        *l.get() = Box::into_raw(Box::new(reactor));
        mem::transmute(*l.get())
    })
}

impl Reactor {
    pub fn run(&'static self) {
        //        for reactor_id in 0..self.smp_queues.smp_count() {
        //            self.smp_queues.submit_to(reactor_id, futures::lazy(|| {
        //                Ok(()) as Result<(), ()>
        //            }));
        //        }

        // Register smp queues poller
        //        if self.smp_queues.smp_count() > 1 {
        //            self.pollers
        // .borrow_mut()
        // .push(Box::new(SmpPollFn::new(self.smp_queues(), self)));
        //        }

        loop {
            if self.check_for_work() {

            } else {

            }
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn log(&self) -> &Logger {
        &self.log
    }

    fn check_for_work(&self) -> bool {
        self.poll_once()
    }

    fn poll_once(&self) -> bool {
        let mut work = false;
        for poller in &*self.pollers.borrow() {
            work |= poller.poll();
        }
        work
    }
}

pub trait PollFn {
    /// Returns true if work was done (false = idle).
    fn poll(&self) -> bool;

    /// Checks if work needs to be done, but without actually doing any
    /// returns true if works needs to be done (false = idle)
    fn pure_poll(&self) -> bool;

    /// Tries to enter interrupt mode.
    ///
    /// If it returns true, then events from this poller will wake
    /// a sleeping idle loop, and exit_interrupt_mode() must be called
    /// to return to normal polling.
    ///
    /// If it returns false, the sleeping idle loop may not be entered.
    fn try_enter_interrupt_mode(&self) -> bool {
        false
    }

    fn exit_interrupt_mode(&self) {}
}
