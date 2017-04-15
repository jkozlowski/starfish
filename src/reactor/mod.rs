use futures::Future;
use smp_message_queue::SmpQueues;
use slog::Logger;
use state::LocalStorage;
use std::fmt;
use std::mem;
use std::vec::Vec;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio_core::reactor::Core;
use util::semaphore::Semaphore;

pub struct Lazy<T>(pub *const T);

pub static mut REACTOR: Lazy<Vec<*const Reactor>> = Lazy(0 as *const Vec<*const Reactor>);

//#[allow(missing_copy_implementations)]
//#[allow(non_camel_case_types)]
//#[allow(dead_code)]
//#[doc = r" This is an example for using doc comment attributes"]
//struct EXAMPLE {
//    __private_field: (),
//}
//#[doc(hidden)]
//static EXAMPLE: EXAMPLE = EXAMPLE{__private_field: (),};
//impl ::__Deref for EXAMPLE {
//    type
//    Target
//    =
//    u8;
//    #[allow(unsafe_code)]
//    fn deref(&self) -> &u8 {
//        unsafe {
//            #[inline(always)]
//            fn __static_ref_initialize() -> u8 { 42 }
//            #[inline(always)]
//            unsafe fn __stability() -> &'static u8 {
//                use std::sync::ONCE_INIT;
//                static mut LAZY: ::lazy::Lazy<u8> =
//                    ::lazy::Lazy(0 as *const u8, ONCE_INIT);
//                LAZY.get(__static_ref_initialize)
//            }
//            __stability()
//        }
//    }
//}
//impl ::LazyStatic for EXAMPLE {
//    fn initialize(lazy: &Self) { let _ = &**lazy; }
//}

pub struct Reactor {
    id: usize,
    //    reactor_backend_epoll _backend;
    backend: Core,
    //    sigset_t _active_sigmask; // holds sigmask while sleeping with sig disabled
    //    std::vector<pollfn*> _pollers;
    //    std::unique_ptr<io_queue> my_io_queue = {};
    //    shard_id _io_coordinator;
    //    io_queue* _io_queue;
    //    std::vector<std::function<future<> ()>> _exit_funcs;
    //    unsigned _id = 0;
    //    bool _stopping = false;
    //    bool _stopped = false;
    //    condition_variable _stop_requested;
    //    bool _handle_sigint = true;
    //    promise<std::unique_ptr<network_stack>> _network_stack_ready_promise;
    //    int _return = 0;
    //    timer_t _steady_clock_timer = {};
    //    timer_t _task_quota_timer = {};
    //    promise<> _start_promise;
    //    semaphore _cpu_started;
    cpu_started: Semaphore,

    //    uint64_t _tasks_processed = 0;
    //    unsigned _max_task_backlog = 1000;
    //    seastar::timer_set<timer<>, &timer<>::_link> _timers;
    //    seastar::timer_set<timer<>, &timer<>::_link>::timer_list_t _expired_timers;
    //    seastar::timer_set<timer<lowres_clock>, &timer<lowres_clock>::_link> _lowres_timers;
    //    seastar::timer_set<timer<lowres_clock>, &timer<lowres_clock>::_link>::timer_list_t _expired_lowres_timers;
    //    io_context_t _io_context;
    //    std::vector<struct ::iocb> _pending_aio;
    //    semaphore _io_context_available;
    //    uint64_t _aio_reads = 0;
    //    uint64_t _aio_read_bytes = 0;
    //    uint64_t _aio_writes = 0;
    //    uint64_t _aio_write_bytes = 0;
    //    uint64_t _fsyncs = 0;
    //    uint64_t _cxx_exceptions = 0;
    //    circular_buffer<std::unique_ptr<task>> _pending_tasks;
    //    circular_buffer<std::unique_ptr<task>> _at_destroy_tasks;
    //    std::chrono::duration<double> _task_quota;
    //    /// Handler that will be called when there is no task to execute on cpu.
    //    /// It represents a low priority work.
    //    ///
    //    /// Handler's return value determines whether handler did any actual work. If no work was done then reactor will go
    //    /// into sleep.
    //    ///
    //    /// Handler's argument is a function that returns true if a task which should be executed on cpu appears or false
    //    /// otherwise. This function should be used by a handler to return early if a task appears.
    //    idle_cpu_handler _idle_cpu_handler{ [] (work_waiting_on_reactor) {return idle_cpu_handler_result::no_more_work;} };
    //    std::unique_ptr<network_stack> _network_stack;
    //    // _lowres_clock will only be created on cpu 0
    //    std::unique_ptr<lowres_clock> _lowres_clock;
    //    lowres_clock::time_point _lowres_next_timeout;
    //    std::experimental::optional<poller> _epoll_poller;
    //    std::experimental::optional<pollable_fd> _aio_eventfd;
    //    const bool _reuseport;
    //    circular_buffer<double> _loads;
    //    double _load = 0;
    //    std::chrono::nanoseconds _max_poll_time = calculate_poll_time();
    //    circular_buffer<output_stream<char>* > _flush_batching;
    //    std::atomic<bool> _sleeping alignas(64);
    //    pthread_t _thread_id alignas(64) = pthread_self();
    //    bool _strict_o_direct = true;
    //    signals _signals;
    //    thread_pool _thread_pool;
    sleeping: Arc<AtomicBool>,
    log: Logger,
    smp_queues: SmpQueues,
}

impl fmt::Debug for Reactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reactor({})", self.id)
    }
}

impl Reactor {
//    #[inline]
//    pub fn with<F, R>(f: F) -> R
//        where F: FnOnce(&Reactor) -> R
//    {
//        REACTOR.with(f)
//    }

    pub fn run(id: usize,
               log: Logger,
               sleeping: Arc<AtomicBool>,
               smp_queues: SmpQueues) {

        let reactor = Reactor::new(id, log, sleeping, smp_queues);

        //        auto collectd_metrics = register_collectd_metrics();
        //
        //    #ifndef HAVE_OSV
        //        poller io_poller(std::make_unique<io_pollfn>(*this));
        //    #endif
        //
        //        poller sig_poller(std::make_unique<signal_pollfn>(*this));
        //        poller aio_poller(std::make_unique<aio_batch_submit_pollfn>(*this));
        //        poller batch_flush_poller(std::make_unique<batch_flush_pollfn>(*this));
        //
        //        start_aio_eventfd_loop();
        //
        //        if (_id == 0) {
        //           if (_handle_sigint) {
        //              _signals.handle_signal_once(SIGINT, [this] { stop(); });
        //           }
        //           _signals.handle_signal_once(SIGTERM, [this] { stop(); });
        //        }
        //

//        let log_1 = log.clone();
//        let cpu_started_fut =
//            cpu_started
//                .wait(smp_queues.smp_count())
//                .and_then(move |_| {
//                    trace!(log_1, "cpu_started");
//                    //  _network_stack->initialize().then([this] {
//                    //      _start_promise.set_value();
//                    //  });
//                    Ok(())
//                });
//
//        let mut backend = Core::new().unwrap();
//        let handle = backend.handle();
//        handle.spawn(cpu_started_fut);
        //        _network_stack_ready_promise.get_future().then([this] (std::unique_ptr<network_stack> stack) {
        //            _network_stack = std::move(stack);
        //            for (unsigned c = 0; c < smp::count; c++) {
        //                smp::submit_to(c, [] {
        //                        engine()._cpu_started.signal();
        //                });
        //            }
        //        });
        //
        //        // Register smp queues poller
        //        std::experimental::optional<poller> smp_poller;
        //        if (smp::count > 1) {
        //            smp_poller = poller(std::make_unique<smp_pollfn>(*this));
        //        }
        //
        //        poller syscall_poller(std::make_unique<syscall_pollfn>(*this));
        //    #ifndef HAVE_OSV
        //        _signals.handle_signal(alarm_signal(), [this] {
        //            complete_timers(_timers, _expired_timers, [this] {
        //                if (!_timers.empty()) {
        //                    enable_timer(_timers.get_next_timeout());
        //                }
        //            });
        //        });
        //    #endif
        //
        //        poller drain_cross_cpu_freelist(std::make_unique<drain_cross_cpu_freelist_pollfn>());
        //
        //        poller expire_lowres_timers(std::make_unique<lowres_timer_pollfn>(*this));
        //
        //        using namespace std::chrono_literals;
        //        timer<lowres_clock> load_timer;
        //        steady_clock_type::rep idle_count = 0;
        //        auto idle_start = steady_clock_type::now(), idle_end = idle_start;
        //        load_timer.set_callback([this, &idle_count, &idle_start, &idle_end] () mutable {
        //            auto load = double(idle_count + (idle_end - idle_start).count()) / double(std::chrono::duration_cast<steady_clock_type::duration>(1s).count());
        //            load = std::min(load, 1.0);
        //            idle_count = 0;
        //            idle_start = idle_end;
        //            _loads.push_front(load);
        //            if (_loads.size() > 5) {
        //                auto drop = _loads.back();
        //                _loads.pop_back();
        //                _load -= (drop/5);
        //            }
        //            _load += (load/5);
        //        });
        //        load_timer.arm_periodic(1s);
        //
        //        itimerspec its = {};
        //        auto nsec = std::chrono::duration_cast<std::chrono::nanoseconds>(_task_quota).count();
        //        auto tv_nsec = nsec % 1'000'000'000;
        //        auto tv_sec = nsec / 1'000'000'000;
        //        its.it_value.tv_nsec = tv_nsec;
        //        its.it_value.tv_sec = tv_sec;
        //        its.it_interval = its.it_value;
        //        auto r = timer_settime(_task_quota_timer, 0, &its, nullptr);
        //        assert(r == 0);
        //
        //        struct sigaction sa_task_quota = {};
        //        sa_task_quota.sa_handler = &reactor::clear_task_quota;
        //        sa_task_quota.sa_flags = SA_RESTART;
        //        r = sigaction(task_quota_signal(), &sa_task_quota, nullptr);
        //        assert(r == 0);
        //
        //        bool idle = false;
        //
        //        std::function<bool()> check_for_work = [this] () {
        //            return poll_once() || !_pending_tasks.empty() || seastar::thread::try_run_one_yielded_thread();
        //        };
        //        std::function<bool()> pure_check_for_work = [this] () {
        //            return pure_poll_once() || !_pending_tasks.empty() || seastar::thread::try_run_one_yielded_thread();
        //        };

//        while true {
//            backend.turn(None);
        //            run_tasks(_pending_tasks);
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
        //            if (check_for_work()) {
        //                if (idle) {
        //                    idle_count += (idle_end - idle_start).count();
        //                    idle_start = idle_end;
        //                    idle = false;
        //                }
        //            } else {
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
        //            }
//        }
        info!(reactor.log, "seems to work: {}", id);
//        })});
        //        // To prevent ordering issues from rising, destroy the I/O queue explicitly at this point.
        //        // This is needed because the reactor is destroyed from the thread_local destructors. If
        //        // the I/O queue happens to use any other infrastructure that is also kept this way (for
        //        // instance, collectd), we will not have any way to guarantee who is destroyed first.
        //        my_io_queue.reset(nullptr);
        //        return _return;
    }

    pub fn new(id: usize,
               log: Logger,
               sleeping: Arc<AtomicBool>,
               smp_queues: SmpQueues)
        -> &'static Reactor
    {
        let reactor = Reactor {
            id: id,
            backend: Core::new().unwrap(),
            cpu_started: Semaphore::new(0),
            log: log,
            sleeping: sleeping,
            smp_queues: smp_queues,
        };

        unsafe {
            let reactors: &mut Vec<*const Reactor> = mem::transmute(REACTOR.0);
            let reactor = Box::into_raw(Box::new(reactor));
            let elem = reactors.get_unchecked_mut(id);
            *elem = reactor;
            mem::transmute(reactor)
        }
    }
}

pub trait PollFn {
    /// Returns true if work was done (false = idle).
    fn poll() -> bool;

    /// Checks if work needs to be done, but without actually doing any
    /// returns true if works needs to be done (false = idle)
    fn pure_poll() -> bool;

    /// Tries to enter interrupt mode.
    ///
    /// If it returns true, then events from this poller will wake
    /// a sleeping idle loop, and exit_interrupt_mode() must be called
    /// to return to normal polling.
    ///
    /// If it returns false, the sleeping idle loop may not be entered.
    fn try_enter_interrupt_mode() -> bool {
        false
    }

    fn exit_interrupt_mode() {}
}
