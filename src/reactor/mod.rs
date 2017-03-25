use smp_message_queue::SmpQueues;
use slog::Logger;
use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

scoped_thread_local!(static REACTOR: Reactor);

pub struct Reactor {
    id: usize,
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
    #[inline]
    pub fn with<F, R>(f: F) -> R
        where F: FnOnce(&Reactor) -> R
    {
        REACTOR.with(f)
    }

    pub fn run(&mut self) {
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
        //        _cpu_started.wait(smp::count).then([this] {
        //            _network_stack->initialize().then([this] {
        //                _start_promise.set_value();
        //            });
        //        });
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
        //        while (true) {
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
        //        // To prevent ordering issues from rising, destroy the I/O queue explicitly at this point.
        //        // This is needed because the reactor is destroyed from the thread_local destructors. If
        //        // the I/O queue happens to use any other infrastructure that is also kept this way (for
        //        // instance, collectd), we will not have any way to guarantee who is destroyed first.
        //        my_io_queue.reset(nullptr);
        //        return _return;
    }

    pub fn allocate_reactor<F, R>(id: usize,
                                  log: Logger,
                                  sleeping: Arc<AtomicBool>,
                                  smp_queues: SmpQueues,
                                  f: F)
        where F: FnOnce(&Reactor) -> R
    {
        let reactor = Reactor {
            id: id,
            log: log,
            sleeping: sleeping,
            smp_queues: smp_queues,
        };
        REACTOR.set(&reactor, || f(&reactor));
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
