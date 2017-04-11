use crossbeam;
use reactor;
use reactor::Reactor;
use sys::imp::reactor_handle::ReactorHandle;
use smp_message_queue::SmpQueues;
use smp_message_queue::make_smp_message_queue;
use slog::Logger;
use std::mem;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Barrier;
use std::sync::mpsc::channel;
use itertools;

pub struct Smp {}

impl Smp {
    pub fn configure(log: Logger) {
        let smp_count: usize = 4;
        let mut all_event_loops_done = None;

        // TODO: mask signals
        // TODO: figure out thread_affinity
        // TODO: figure out nr_cpus
        // TODO: figure out memory layout and cpu configuration

        let reactors_registered = Barrier::new(smp_count);
        let smp_queues_constructed = Barrier::new(smp_count);
        let inited = Barrier::new(smp_count);

        // TODO: allocate io queues and assign coordinators

        mem::replace(&mut all_event_loops_done, Some(Barrier::new(smp_count)));

        crossbeam::scope(|scope| {
            let log = log.clone();
            let mut reactor_receives = Vec::with_capacity(smp_count);
            let mut queues_publishes = Vec::with_capacity(smp_count);

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
                let log = log.clone();

                scope.spawn(move || {
                    let mut other_reactor = OtherReactor {};
                    Smp::configure_single_reactor(log,
                                                  reactor_id,
                                                  &mut other_reactor,
                                                  reactor_registered,
                                                  smp_queue_constructed,
                                                  init,
                                                  reactor_publish,
                                                  queue_receive)
                });
            }

            let mut reactor_zero = Reactor0 {
                smp_count: smp_count,
                reactor_receives: &reactor_receives,
                queue_senders: &queues_publishes,
            };
            Smp::configure_single_reactor(log,
                                          0,
                                          &mut reactor_zero,
                                          &reactors_registered,
                                          &smp_queues_constructed,
                                          &inited,
                                          smp_0_reactor_publish,
                                          smp_0_queue_receive);
        });
    }

    fn configure_single_reactor(root: Logger,
                                reactor_id: usize,
                                reactor_init: &mut ReactorInit,
                                reactor_registered: &Barrier,
                                smp_queue_constructed: &Barrier,
                                init: &Barrier,
                                reactor_publish: Sender<ReactorHandle>,
                                queue_receive: Receiver<SmpQueues>) {
        let log = root.new(o!("reactor_id" => reactor_id));
        trace!(log, "started");

        let sleeping = Arc::new(AtomicBool::new(false));
        reactor_publish.send(ReactorHandle::new(sleeping.clone())).unwrap();

        reactor_registered.wait();
        trace!(log, "Reactor registered");

        reactor_init.on_reactor_registered();

        smp_queue_constructed.wait();
        trace!(log, "Smp queue constructed");

        let smp_queue = queue_receive.recv().expect("Expected SmpQueue");

//        reactor::REACTOR.set(move || {
//            Reactor::new(reactor_id, log.clone(), sleeping.clone(), smp_queue)
//        });

//        reactor::REACTOR.get();

//        trace!(log, "Reactor created");

        // start_all_queues();
        // assign_io_queue(i, queue_idx);
        init.wait();

        // engine().configure(configuration);
        reactor::Reactor::run(reactor_id, log.clone(), sleeping.clone(), smp_queue);
    }
}

trait ReactorInit {
    fn on_reactor_registered(&mut self);
}

struct Reactor0<'a> {
    smp_count: usize,
    reactor_receives: &'a Vec<Receiver<ReactorHandle>>,
    queue_senders: &'a Vec<Sender<SmpQueues>>,
}

impl<'a> ReactorInit for Reactor0<'a> {
    fn on_reactor_registered(&mut self) {
        let mut reactors = Vec::with_capacity(self.smp_count);

        for rx in self.reactor_receives {
            let reactor = rx.recv().unwrap();
            reactors.push(reactor);
        }

        assert!(reactors.len() == self.smp_count);

        // REALLY SHADY STUFF HERE
        let mut all_producers = Vec::with_capacity(self.smp_count);
        let mut all_consumers = Vec::with_capacity(self.smp_count);
        {
            for _ in 0..self.smp_count {
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
            for i in 0..self.smp_count {
                for j in 0..self.smp_count {
                    let (p, c) = {
                        let from = reactors[i].clone();
                        let to = reactors[j].clone();
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

        for (p, c, s, i) in itertools::multizip((all_producers,
                                                 all_consumers,
                                                 self.queue_senders,
                                                 0..)) {
            s.send(SmpQueues::new(p, c, i, self.smp_count)).unwrap();
        }
    }
}

struct OtherReactor {}

impl ReactorInit for OtherReactor {
    fn on_reactor_registered(&mut self) {}
}
