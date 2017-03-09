// https://github.com/rust-lang/rust/blob/4b40bc85cbc1d072179c92ce01655db0272aa598/src/libstd/io/stdio.rs#L215-L245
// https://doc.rust-lang.org/1.0.0/std/thread/fn.scoped.html
// https://doc.rust-lang.org/std/sync/struct.Barrier.html
// https://doc.rust-lang.org/std/cell/index.html
// https://users.rust-lang.org/t/rust-thread-local-bad-performance/4385
// https://github.com/rust-lang/rust/issues/27779
// https://play.rust-lang.org/?gist=1560082065f1cafffd14&version=nightly
// https://gist.github.com/Connorcpu/3dc6233bd59522f0b6d650e90d781c63
// http://stackoverflow.com/questions/32750829/passing-a-reference-to-a-stack-variable-to-a-scoped-thread
// http://blog.ezyang.com/2013/12/two-bugs-in-the-borrow-checker-every-rust-developer-should-know-about/
// https://doc.rust-lang.org/src/std/sync/once.rs.html#139-329

//static std::vector<posix_thread> _threads;
//static std::experimental::optional<boost::barrier> _all_event_loops_done;
//static std::vector<reactor*> _reactors;
//static smp_message_queue** _qs;
//static std::thread::id _tmain;
//        static boost::barrier reactors_registered(smp::count);
//        static boost::barrier smp_queues_constructed(smp::count);
//        static boost::barrier inited(smp::count);

/// This is a whoooole bunch of unsafe code!

use core::nonzero::NonZero;
use core::ops::Deref;
use crossbeam;
use crossbeam::Scope;
use crossbeam::ScopedJoinHandle;
use slab::Slab;
use smp_message_queue::SmpQueues;
use smp_message_queue::make_smp_message_queue;
use state::LocalStorage;
use state::Storage;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::marker::PhantomData;
use std::mem;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::ptr;
use std::ptr::Unique;
use std::sync::Barrier;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::marker::Copy;
use thread_scoped;
use itertools;

use std::thread;
use std::time::Duration;

scoped_thread_local!(static REACTOR: Reactor);
scoped_thread_local!(static SMP_QUEUES: SmpQueues);

#[derive(Debug)]
pub struct Reactor {
    id: usize,
    val: usize
    //    sleeping: AtomicBool
}

pub struct WakeupHandle<'a> {
    sleeping: &'a AtomicBool
}

#[derive(Clone, Copy)]
pub struct UnsafePtr<T> {
    ptr: NonZero<*const T>,
    _marker: PhantomData<T>,
}

impl<T> UnsafePtr<T> {
    pub unsafe fn new(t: &T) -> UnsafePtr<T> {
        UnsafePtr {
            ptr: unsafe { NonZero::new(t as *const T) },
            _marker: PhantomData
        }
    }

    pub fn cp(&self) -> UnsafePtr<T> {
        UnsafePtr {
            ptr: unsafe { self.ptr },
            _marker: PhantomData
        }
    }
}

/// Just trust me, ok
unsafe impl<T> Send for UnsafePtr<T> {}

unsafe impl<T> Sync for UnsafePtr<T> {}

impl Reactor {
    #[inline]
    pub fn with<F, R>(f: F) -> R where F: FnOnce(&Reactor) -> R {
        REACTOR.with(f)
    }

    pub fn allocate_reactor<F, R>(id: usize, f: F) where F: FnOnce(&Reactor) -> R {
        let reactor = Reactor {
            id: id,
            val: id + 1,
            //            sleeping: AtomicBool::new(false)
        };
        REACTOR.set(&reactor, || f(&reactor));
    }
}

pub struct Smp {}

impl Smp {
    // TODO:
    // # signals
    pub fn configure() {
        let smp_count: usize = 4;
        let mut all_event_loops_done = None;
        let reactors_storage: Storage<Vec<UnsafePtr<Reactor>>> = Storage::new();

        // TODO: mask signals
        // TODO: figure out thread_affinity
        // TODO: figure out nr_cpus

        // TODO: figure out memory layout and cpu configuration

        //  // Better to put it into the smp class, but at smp construction time
        //  // correct smp::count is not known.
        //  static boost::barrier reactors_registered(smp::count);
        let reactors_registered = Barrier::new(smp_count);
        //  static boost::barrier smp_queues_constructed(smp::count);
        let smp_queues_constructed = Barrier::new(smp_count);
        //  static boost::barrier inited(smp::count);
        let inited = Barrier::new(smp_count);

        // TODO: allocate io queues and assign coordinators

        mem::replace(&mut all_event_loops_done, Some(Barrier::new(smp_count)));

        crossbeam::scope(|scope| {
            let mut reactor_receives = Vec::with_capacity(smp_count);
            let mut queues_publishes = Vec::with_capacity(smp_count);

            // What a copout!
            let (smp_0_reactor_publish, smp_0_reactor_receive) = channel();
            reactor_receives.push(smp_0_reactor_receive);

            let (smp_0_queue_publish, smp_0_queue_receive) = channel();
            queues_publishes.push(smp_0_queue_publish);

            for reactor_id in 1..smp_count {
                let (reactor_publish, reactor_receive) = channel();
                reactor_receives.push(reactor_receive);

                let (queue_publish, queue_receive) = channel();
                queues_publishes.push(queue_publish);

                let reactor_registered = &reactors_registered;
                let smp_queue_constructed = &smp_queues_constructed;
                let init = &inited;
                let reactor_storage = &reactors_storage;

                scope.spawn(move || {
                    let mut other_reactor = OtherReactor {};
                    Smp::configure_single_reactor(reactor_id,
                                                  &mut other_reactor,
                                                  reactor_registered,
                                                  smp_queue_constructed,
                                                  init,
                                                  reactor_publish,
                                                  queue_receive,
                                                  reactor_storage)});
            }

            let mut reactor_zero = Reactor0 {
                smp_count: smp_count,
                reactor_receives: &reactor_receives,
                queue_senders: &queues_publishes,
                reactors_storage: &reactors_storage
            };
            Smp::configure_single_reactor(0,
                                          &mut reactor_zero,
                                          &reactors_registered,
                                          &smp_queues_constructed,
                                          &inited,
                                          smp_0_reactor_publish,
                                          smp_0_queue_receive,
                                          &reactors_storage);
        });
    }

    fn configure_single_reactor(
        reactor_id: usize,
        reactor_init: &mut ReactorInit,
        reactor_registered: &Barrier,
        smp_queue_constructed: &Barrier,
        init: &Barrier,
        reactor_publish: Sender<UnsafePtr<Reactor>>,
        queue_receive: Receiver<SmpQueues>,
        reactor_storage: &Storage<Vec<UnsafePtr<Reactor>>>)
    {
        Reactor::allocate_reactor(reactor_id, |r| {
            trace!("Thread [{:?}]: started; {:?}", reactor_id, r as *const _);
            reactor_publish.send(unsafe { UnsafePtr::new(r) }).unwrap();

            reactor_registered.wait();
            info!("Thread [{:?}]: Reactor registered", reactor_id);

            reactor_init.on_reactor_registered(r);

            smp_queue_constructed.wait();
            info!("Thread [{:?}]: Smp queue constructed", reactor_id);

            let smp_queue = queue_receive.recv().expect("Expected SmpQueue");
            SMP_QUEUES.set(&smp_queue, || {
                info!("Thread [{:?}]: Smp queues setup: {:?}", reactor_id, smp_queue.reactor_id());

                for ref reactor in reactor_storage.get() {
                    info!("Thread [{:?}]: {:?}: {:?}", reactor_id, reactor.ptr, unsafe { &**(reactor.ptr) });
                }
                // start_all_queues();
                // assign_io_queue(i, queue_idx);
                init.wait();

                // engine().configure(configuration);
                // engine().run();
            });
        })
    }
}

trait ReactorInit {
    fn on_reactor_registered(&mut self, r: &Reactor);
}

struct Reactor0<'a> {
    smp_count: usize,
    reactor_receives: &'a Vec<Receiver<UnsafePtr<Reactor>>>,
    queue_senders: &'a Vec<Sender<SmpQueues>>,
    reactors_storage: &'a Storage<Vec<UnsafePtr<Reactor>>>
}

impl<'a> ReactorInit for Reactor0<'a> {
    fn on_reactor_registered(&mut self, r: &Reactor) {
        let mut reactors = Vec::with_capacity(self.smp_count);

        for rx in self.reactor_receives {
            let reactor = rx.recv().unwrap();
            reactors.push(reactor);
        }

        assert!(reactors.len() == self.smp_count);

        self.reactors_storage.set(reactors);

        // REALLY SHADY STUFF HERE
        let mut all_producers = Vec::with_capacity(self.smp_count);
        let mut all_consumers = Vec::with_capacity(self.smp_count);
        {
            for i in 0..self.smp_count {
                let mut pair_producers = Vec::with_capacity(self.smp_count);
                unsafe { pair_producers.set_len(self.smp_count) }
                let mut pair_consumers = Vec::with_capacity(self.smp_count);
                unsafe { pair_consumers.set_len(self.smp_count) }

                assert!(pair_producers.len() == self.smp_count);
                assert!(pair_consumers.len() == self.smp_count);

                all_producers.push(pair_producers);
                all_consumers.push(pair_consumers);
            }

            // Really shady stuff here...
            let rs = self.reactors_storage.get();
            for i in 0..self.smp_count {
                for j in 0..self.smp_count {
                    let (p, c) = unsafe {
                        let from = rs[i].cp();
                        let to = rs[j].cp();
                        make_smp_message_queue(from, to)
                    };
                    unsafe {
                        let b = all_producers[i].get_unchecked_mut(j);
                        ptr::write(b, p);
                    }
                    unsafe {
                        let b = all_consumers[j].get_unchecked_mut(i);
                        ptr::write(b, c);
                    }
                }
            }
        }

        assert!(all_producers.len() == self.smp_count);
        assert!(all_consumers.len() == self.smp_count);

        for (p, c, s, i) in itertools::multizip((all_producers, all_consumers, self.queue_senders, 0..)) {
            s.send(SmpQueues::new(p, c, i));
        }
    }
}

struct OtherReactor {
}

impl ReactorInit for OtherReactor {
    fn on_reactor_registered(&mut self, r: &Reactor) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;
    use std::thread;
    use std::time::Duration;
    use std::ptr;

    #[test]
    fn it_works() {
        env_logger::init().unwrap();
        Smp::configure();
    }
}
