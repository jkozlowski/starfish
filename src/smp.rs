use crossbeam;
use reactor;
use reactor::Reactor;
use sys::imp::reactor_handle::ReactorHandle;
use smp_message_queue::SmpQueues;
use smp_message_queue::Channel;
use smp_message_queue::make_channel_pair;
use slog::Logger;
use std::mem;
use std::option::Option;
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

        //        unsafe {
        //            let mut reactors: Vec<*const Reactor> = Vec::new();
        //            {
        //                reactors.resize(smp_count, 0 as *const Reactor);
        //            }
        //            let r = &mut reactor::REACTOR.0;
        //            *r = Box::into_raw(Box::new(reactors));
        //        }

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
                    let reactor = Smp::configure_single_reactor(log,
                                                                reactor_id,
                                                                &mut other_reactor,
                                                                reactor_registered,
                                                                smp_queue_constructed,
                                                                init,
                                                                reactor_publish,
                                                                queue_receive);
                    reactor.run();
                });
            }

            let mut reactor_zero = Reactor0 {
                smp_count: smp_count,
                reactor_receives: &reactor_receives,
                queue_senders: &queues_publishes,
            };
            let reactor = Smp::configure_single_reactor(log,
                                                        0,
                                                        &mut reactor_zero,
                                                        &reactors_registered,
                                                        &smp_queues_constructed,
                                                        &inited,
                                                        smp_0_reactor_publish,
                                                        smp_0_queue_receive);
            reactor.run();
        });
    }

    fn configure_single_reactor(root: Logger,
                                reactor_id: usize,
                                reactor_init: &mut ReactorInit,
                                reactor_registered: &Barrier,
                                smp_queue_constructed: &Barrier,
                                init: &Barrier,
                                reactor_publish: Sender<ReactorHandle>,
                                queue_receive: Receiver<SmpQueues>)
                                -> &'static mut Reactor {
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

        trace!(log, "Reactor created");

        // start_all_queues();
        // assign_io_queue(i, queue_idx);
        init.wait();

        // engine().configure(configuration);
        reactor::create_reactor(reactor_id, log.clone(), sleeping.clone(), smp_queue)
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
        let mut all_channels: Vec<Vec<Option<Channel>>> = Vec::with_capacity(self.smp_count);
        {
            for _ in 0..self.smp_count {
                let mut channels = Vec::with_capacity(self.smp_count);
                unsafe { channels.set_len(self.smp_count) }
                all_channels.push(channels);
            }

            // Really shady stuff here...
            for i in 0..self.smp_count {
                for j in 0..self.smp_count {

                    let not_happened = unsafe {
                        all_channels[i].get_unchecked(j).is_none()
                    };

                    if not_happened {
                        let (c_i, c_j) = {
                            let from = reactors[i].clone();
                            let to = reactors[j].clone();
                            make_channel_pair(from, to)
                        };

                        unsafe {
                            let b = all_channels[i].get_unchecked_mut(j);
                            mem::replace(b, Some(c_i));
                        }

                        unsafe {
                            let b = all_channels[j].get_unchecked_mut(i);
                            mem::replace(b, Some(c_j));
                        }
                    }
                }
            }
        }

        assert!(all_channels.len() == self.smp_count);

        for (cs, s, i) in itertools::multizip((all_channels,
                                               self.queue_senders,
                                               0..)) {
            let channels = cs.into_iter().map(|x| x.unwrap()).collect();
            s.send(SmpQueues::new(i, self.smp_count, channels)).unwrap();
        }
    }
}

struct OtherReactor {}

impl ReactorInit for OtherReactor {
    fn on_reactor_registered(&mut self) {}
}
