use futures;
use futures::Future;
use smp_message_queue::SmpQueues;
use smp_message_queue::SmpPollFn;
use slog::Logger;
use std::cell::UnsafeCell;
use std::cell::RefCell;
use std::fmt;
use std::mem;
use std::ptr::null;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio_core::reactor::Core;
use tokio_core::reactor::Handle;
//use util::semaphore::Semaphore;

thread_local! {
    static REACTOR: UnsafeCell<*const Reactor> = UnsafeCell::new(null());
}

pub struct Reactor {
    id: usize,
    backend: RefCell<Core>,
    handle: Handle,
    pollers: RefCell<Vec<Box<PollFn>>>,
//    started: Semaphore,
//    cpu_started: Semaphore,
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

pub fn create_reactor(id: usize,
                      log: Logger,
                      sleeping: Arc<AtomicBool>,
                      smp_queues: SmpQueues)
                      -> &'static mut Reactor {
    let core = Core::new().unwrap();
    let handle = core.handle();
    let reactor = Reactor {
        id: id,
        backend: RefCell::new(core),
        handle: handle,
        pollers: RefCell::new(Vec::new()),
//        started: Semaphore::new(0),
//        cpu_started: Semaphore::new(0),
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
//        let cpu_started_fut =
//            self.cpu_started.wait(self.smp_queues.smp_count()).and_then(move |_| {
//                //  _network_stack->initialize().then([this] {
//                local().started.signal(1);
//                //      _start_promise.set_value();
//                //  });
//                Ok(())
//            });
//        self.spawn(cpu_started_fut);

        for reactor_id in 0..self.smp_queues.smp_count() {
            self.smp_queues.submit_to(reactor_id, futures::lazy(|| {
//                local().cpu_started.signal(1);
                Ok(()) as Result<(), ()> // Required for inference
            }));
        }

        // Register smp queues poller
        if self.smp_queues.smp_count() > 1 {
            self.pollers.borrow_mut().push(Box::new(SmpPollFn::new(self.smp_queues(), self)));
        }

        loop {
            self.backend.borrow_mut().turn(Some(Duration::from_millis(1)));
            //            if (_stopped) {
            //                load_timer.cancel();
            //                // Final tasks may include sending the last response to cpu 0, so run them
            //                while (!_pending_tasks.empty()) {
            //                    run_tasks(_pending_tasks);
            //                }
            //                while (!_at_destroy_tasks.empty()) {
            //                    run_tasks(_at_destroy_tasks);
            //                }
            //                smp::arrive_at_event_loop_end();
            //                if (_id == 0) {
            //                    smp::join_all();
            //                }
            //                break;
            //            }
            //
            if self.check_for_work() {
                //                if (idle) {
                //                    idle_count += (idle_end - idle_start).count();
                //                    idle_start = idle_end;
                //                    idle = false;
                //                }
            } else {
                //                idle_end = steady_clock_type::now();
                //                if (!idle) {
                //                    idle_start = idle_end;
                //                    idle = true;
                //                }
                //                bool go_to_sleep = true;
                //                try {
                //                    // we can't run check_for_work(), because that can run tasks in the context
                //                    // of the idle handler which change its state, without the idle handler expecting
                //                    // it.  So run pure_check_for_work() instead.
                //                    auto handler_result = _idle_cpu_handler(pure_check_for_work);
                //                    go_to_sleep = handler_result == idle_cpu_handler_result::no_more_work;
                //                } catch (...) {
                //                    report_exception("Exception while running idle cpu handler", std::current_exception());
                //                }
                //                if (go_to_sleep) {
                //                    _mm_pause();
                //                    if (idle_end - idle_start > _max_poll_time) {
                //                        sleep();
                //                        // We may have slept for a while, so freshen idle_end
                //                        idle_end = steady_clock_type::now();
                //                    }
                //                } else {
                //                    // We previously ran pure_check_for_work(), might not actually have performed
                //                    // any work.
                //                    check_for_work();
                //                }
            }
        }
        //        })});
        //        // To prevent ordering issues from rising, destroy the I/O queue explicitly at this point.
        //        // This is needed because the reactor is destroyed from the thread_local destructors. If
        //        // the I/O queue happens to use any other infrastructure that is also kept this way (for
        //        // instance, collectd), we will not have any way to guarantee who is destroyed first.
        //        my_io_queue.reset(nullptr);
        //        return _return;
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn log(&self) -> &Logger {
        &self.log
    }

    pub fn smp_queues(&self) -> &SmpQueues {
        &self.smp_queues
    }

//    pub fn when_started(&self) -> impl Future<Item = (), Error = ()> {
//        self.started.wait(1)
//    }

    pub fn spawn<F>(&self, f: F)
        where F: Future<Item = (), Error = ()> + 'static
    {
        self.handle.spawn(f)
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
