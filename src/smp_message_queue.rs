use bounded_spsc_queue;
use bounded_spsc_queue::{Producer, Consumer};
use futures::Future;
use futures::unsync::oneshot::channel;
use futures::unsync::oneshot::Sender;
use futures::unsync::oneshot::Receiver;
use reactor;
use reactor::PollFn;
use reactor::Reactor;
use std::panic::AssertUnwindSafe;
use sys::imp::reactor_handle::ReactorHandle;

// 1) Do not do cancellation for the time being.

pub trait AsyncItem: Send {
    //    future<> process() {}
    //    complete();
    //    auto nr = process_queue<prefetch_cnt>(_pending, [this] (work_item* wi) {
    //        wi->process().then([this, wi] {
    //            respond(wi);
    //        });
    //    });

    //    void smp_message_queue::respond(work_item* item) {
    //    _completed_fifo.push_back(item);
    //    if (_completed_fifo.size() >= batch_size || engine()._stopped) {
    //        flush_response_batch();
    //    }
    //}
}

//
struct AsyncMessage<F, R> {
    fut: F,

    // This is where the remote reactor will write back the value.
    result: Option<R>,

    // Should only be touched back on the sending reactor!
    sender: Sender<R>,
}

unsafe impl<F, R> Send for AsyncMessage<F, R> {}

impl<R, E, F: Future<Item = R, Error = E> + Send> AsyncItem for AsyncMessage<F, R> {}

type BoxMessage = Box<AsyncItem>;

// Producer for posting work items and consumer for getting completions back.
pub struct RequestQueue {
    remote: ReactorHandle,
    // To notify when posting items
    producer: Producer<BoxMessage>,
    consumer: Consumer<BoxMessage>,
}

impl RequestQueue {
    pub fn new(remote: ReactorHandle,
               producer: Producer<BoxMessage>,
               consumer: Consumer<BoxMessage>)
               -> RequestQueue {
        RequestQueue {
            remote: remote,
            producer: producer,
            consumer: consumer,
        }
    }

    pub fn submit<F>(&self, f: F) -> Receiver<Result<F::Item, F::Error>>
        where F: Future + Send + 'static,
              F::Item: Send + 'static,
              F::Error: Send + 'static
    {
        let (sender, rcv) = channel();
        let async_message = AsyncMessage {
            fut: AssertUnwindSafe(f).catch_unwind(),
            result: None,
            sender: sender,
        };

        // TODO: there should be an intermediate queue
        // TODO: need to notify consumer

        // TODO: This can block, so really we should implement this in the future,
        // TODO: so we can wait.

        self.producer.push(Box::new(async_message));
        rcv
    }
}

// Consumer for getting work items and producer for posting completions back.
pub struct WorkQueue {
    remote: ReactorHandle,
    // To notify when posting items
    consumer: Consumer<BoxMessage>,
    producer: Producer<BoxMessage>,
}

impl WorkQueue {
    pub fn new(remote: ReactorHandle,
               consumer: Consumer<BoxMessage>,
               producer: Producer<BoxMessage>)
               -> WorkQueue {
        WorkQueue {
            remote: remote,
            consumer: consumer,
            producer: producer,
        }
    }
}

pub struct Channel {
    // outgoing requests
    request_queue: RequestQueue,
    // incoming requests
    work_queue: WorkQueue,
}

impl Channel {
    fn new(request_queue: RequestQueue,
           work_queue: WorkQueue)
           -> Channel {
        Channel {
            request_queue: request_queue,
            work_queue: work_queue,
        }
    }
}

const QUEUE_LENGTH: usize = 128;

pub fn make_channel_pair(from: ReactorHandle,
                         to: ReactorHandle)
                         -> (Channel, Channel) {

    let from_to_requests = bounded_spsc_queue::make(QUEUE_LENGTH);
    let from_to_completions = bounded_spsc_queue::make(QUEUE_LENGTH);
    let to_from_requests = bounded_spsc_queue::make(QUEUE_LENGTH);
    let to_from_completions = bounded_spsc_queue::make(QUEUE_LENGTH);

    let from_requests = RequestQueue::new(to.clone(), from_to_requests.0, from_to_completions.1);
    let to_work = WorkQueue::new(from.clone(), from_to_requests.1, from_to_completions.0);

    let to_requests = RequestQueue::new(from.clone(), to_from_requests.0, to_from_completions.1);
    let from_work = WorkQueue::new(to.clone(), to_from_requests.1, to_from_completions.0);

    (Channel::new(from_requests, from_work), Channel::new(to_requests, to_work))
}

pub struct SmpQueues {
    reactor_id: usize,
    smp_count: usize,
    queues: Vec<Channel>,
}

impl SmpQueues {
    pub fn new(reactor_id: usize, smp_count: usize, queues: Vec<Channel>) -> SmpQueues {
        assert!(queues.len() == smp_count,
                "queues.len: expected {}, found {}",
                smp_count,
                queues.len());
        SmpQueues {
            reactor_id: reactor_id,
            smp_count: smp_count,
            queues: queues,
        }
    }

    pub fn reactor_id(&self) -> usize {
        self.reactor_id
    }

    pub fn smp_count(&self) -> usize {
        self.smp_count
    }

    pub fn submit_to<F>(&self, reactor_id: usize, f: F) -> Receiver<Result<F::Item, F::Error>>
        where F: Future + Send + 'static,
              F::Item: Send + 'static,
              F::Error: Send + 'static
    {
        if reactor_id == self.reactor_id {
            let (sender, rcv) = channel();
            reactor::local().spawn(f.then(|r| {
                                              // TODO: handle failure
                                              sender.send(r);
                                              Ok(())
                                          }));
            rcv
        } else {
            self.queues[reactor_id].request_queue.submit(f)
        }
    }

    pub fn poll_queues(&self, reactor: &Reactor) -> bool {
        let mut got: usize = 0;
        for i in 0..self.smp_count {
            if reactor.id() != i {
                //let rxq = self.consumers.get(i);
                //                rxq.flush_response_batch();
                //                got += rxq.has_unflushed_responses();
                //                got += rxq.process_incoming();
                //                auto& txq = _qs[i][engine()._id];
                //                txq.flush_request_batch();
                //                got += txq.process_completions();
            }
        }
        got != 0
    }

    pub fn pure_poll_queues(&self, reactor: &Reactor) -> bool {
        false
    }

    fn process_incoming(&self, reactor: &Reactor) {}

    //    void start(unsigned cpuid);
    //    template<size_t PrefetchCnt, typename Func>
    //    size_t process_queue(lf_queue& q, Func process);
    //    size_t process_incoming();
    //    size_t process_completions();
    //    private:
    //    void work();
    //    void submit_item(work_item* wi);
    //    void respond(work_item* wi);
    //    void move_pending();
    //    void flush_request_batch();
    //    void flush_response_batch();
    //    bool has_unflushed_responses() const;
    //    bool pure_poll_rx() const;
    //    bool pure_poll_tx() const;
}

pub struct SmpPollFn<'a> {
    smp_queues: &'a SmpQueues,
    reactor: &'a Reactor,
}

impl<'a> PollFn for SmpPollFn<'a> {
    fn poll(&self) -> bool {
        self.smp_queues.poll_queues(self.reactor)
    }

    fn pure_poll(&self) -> bool {
        self.smp_queues.pure_poll_queues(self.reactor)
    }
}

impl<'a> SmpPollFn<'a> {
    pub fn new<'b>(smp_queues: &'b SmpQueues, reactor: &'b Reactor) -> SmpPollFn<'b> {
        SmpPollFn {
            smp_queues: smp_queues,
            reactor: reactor,
        }
    }
}

//use bounded_spsc_queue;
//use bounded_spsc_queue::{Producer, Consumer};
//use futures::Future;
//use futures::unsync::oneshot::channel;
//use futures::unsync::oneshot::Sender;
//use futures::unsync::oneshot::Receiver;
//use reactor;
//use reactor::PollFn;
//use reactor::Reactor;
//use std::panic::AssertUnwindSafe;
//use sys::imp::reactor_handle::ReactorHandle;
//
//const QUEUE_LENGTH: usize = 128;
//const BATCH_SIZE: usize = 16;
//
//// 1) Do not do cancellation for the time being.
//
//pub trait AsyncItem: Send {
//    //    future<> process() {}
//    //    complete();
//    //    auto nr = process_queue<prefetch_cnt>(_pending, [this] (work_item* wi) {
//    //        wi->process().then([this, wi] {
//    //            respond(wi);
//    //        });
//    //    });
//
//    //    void smp_message_queue::respond(work_item* item) {
//    //    _completed_fifo.push_back(item);
//    //    if (_completed_fifo.size() >= batch_size || engine()._stopped) {
//    //        flush_response_batch();
//    //    }
//    //}
//}
//
////
//struct AsyncMessage<F, R> {
//    fut: F,
//
//    // This is where the remote reactor will write back the value.
//    result: Option<R>,
//
//    // Should only be touched back on the sending reactor!
//    sender: Sender<R>,
//}
//
//unsafe impl<F, R> Send for AsyncMessage<F, R> {}
//
//impl<R, E, F: Future<Item = R, Error = E> + Send> AsyncItem for AsyncMessage<F, R> {}
//
//type BoxMessage = Box<AsyncItem>;
//
//// Producer for posting work items and consumer for getting completions back.
//pub struct RequestQueue {
//    // To notify when posting items
//    remote: ReactorHandle,
//    pending: Vec<BoxMessage>,
//    producer: Producer<BoxMessage>,
//    consumer: Consumer<BoxMessage>,
//    // Stats
//    sent: usize,
//    last_sent_batch: usize
//}
//
//impl RequestQueue {
//    pub fn new(remote: ReactorHandle,
//               producer: Producer<BoxMessage>,
//               consumer: Consumer<BoxMessage>)
//               -> RequestQueue {
//        RequestQueue {
//            remote: remote,
//            pending: Vec::with_capacity(BATCH_SIZE),
//            producer: producer,
//            consumer: consumer,
//            sent: 0,
//            last_sent_batch: 0
//        }
//    }
//
//    pub fn submit<F>(&mut self, f: F) -> Receiver<Result<F::Item, F::Error>>
//        where F: Future + Send + 'static,
//              F::Item: Send + 'static,
//              F::Error: Send + 'static
//    {
//        let (sender, rcv) = channel();
//        let async_message = AsyncMessage {
//            fut: AssertUnwindSafe(f).catch_unwind(),
//            result: None,
//            sender: sender,
//        };
//
//        // TODO: there should be an intermediate queue
//        // TODO: need to notify consumer
//
//        // TODO: This can block, so really we should implement this in the future,
//        // TODO: so we can wait.
//        self.pending.push(Box::new(async_message));
//        if self.pending.len() >= BATCH_SIZE {
//            self.move_pending();
//        }
//
//        rcv
//    }
//
//    fn move_pending(&mut self) {
//        if self.pending.is_empty() {
//            return
//        }
//
//        // push all items: what if not enough space
//        // Need the batch push API
//        let nr = self.pending.len();
//        //        self.producer.push();
//
//        self.remote.maybe_wakeup();
//
//        self.pending.clear();
//        self.last_sent_batch = nr;
//        self.sent += nr;
//    }
//}
//
//// Consumer for getting work items and producer for posting completions back.
//pub struct WorkQueue {
//    remote: ReactorHandle,
//    // To notify when posting items
//    consumer: Consumer<BoxMessage>,
//    producer: Producer<BoxMessage>,
//}
//
//impl WorkQueue {
//    pub fn new(remote: ReactorHandle,
//               consumer: Consumer<BoxMessage>,
//               producer: Producer<BoxMessage>)
//               -> WorkQueue {
//        WorkQueue {
//            remote: remote,
//            consumer: consumer,
//            producer: producer,
//        }
//    }
//}
//
//pub struct Channel {
//    // outgoing requests
//    request_queue: RequestQueue,
//    // incoming requests
//    work_queue: WorkQueue,
//}
//
//impl Channel {
//    fn new(request_queue: RequestQueue,
//           work_queue: WorkQueue)
//           -> Channel {
//        Channel {
//            request_queue: request_queue,
//            work_queue: work_queue,
//        }
//    }
//}
//
//pub fn make_channel_pair(from: ReactorHandle,
//                         to: ReactorHandle)
//                         -> (Channel, Channel) {
//
//    let from_to_requests = bounded_spsc_queue::make(QUEUE_LENGTH);
//    let from_to_completions = bounded_spsc_queue::make(QUEUE_LENGTH);
//    let to_from_requests = bounded_spsc_queue::make(QUEUE_LENGTH);
//    let to_from_completions = bounded_spsc_queue::make(QUEUE_LENGTH);
//
//    let from_requests = RequestQueue::new(to.clone(), from_to_requests.0, from_to_completions.1);
//    let to_work = WorkQueue::new(from.clone(), from_to_requests.1, from_to_completions.0);
//
//    let to_requests = RequestQueue::new(from.clone(), to_from_requests.0, to_from_completions.1);
//    let from_work = WorkQueue::new(to.clone(), to_from_requests.1, to_from_completions.0);
//
//    (Channel::new(from_requests, from_work), Channel::new(to_requests, to_work))
//}
//
//pub struct SmpQueues {
//    reactor_id: usize,
//    smp_count: usize,
//    queues: Vec<Channel>,
//}
//
//impl SmpQueues {
//    pub fn new(reactor_id: usize, smp_count: usize, queues: Vec<Channel>) -> SmpQueues {
//        assert!(queues.len() == smp_count,
//                "queues.len: expected {}, found {}",
//                smp_count,
//                queues.len());
//        SmpQueues {
//            reactor_id: reactor_id,
//            smp_count: smp_count,
//            queues: queues,
//        }
//    }
//
//    pub fn reactor_id(&self) -> usize {
//        self.reactor_id
//    }
//
//    pub fn smp_count(&self) -> usize {
//        self.smp_count
//    }
//
//    pub fn submit_to<F>(&mut self, reactor_id: usize, f: F) -> Receiver<Result<F::Item, F::Error>>
//        where F: Future + Send + 'static,
//              F::Item: Send + 'static,
//              F::Error: Send + 'static
//    {
//        if reactor_id == self.reactor_id {
//            let (sender, rcv) = channel();
//            reactor::local().spawn(f.then(|r| {
//                // TODO: handle failure
//                sender.send(r);
//                Ok(())
//            }));
//            rcv
//        } else {
//            self.queues[reactor_id].request_queue.submit(f)
//        }
//    }
//
//    pub fn poll_queues(&self, reactor: &Reactor) -> bool {
//        let mut got: usize = 0;
//        for i in 0..self.smp_count {
//            if reactor.id() != i {
//                //let rxq = self.consumers.get(i);
//                //                rxq.flush_response_batch();
//                //                got += rxq.has_unflushed_responses();
//                //                got += rxq.process_incoming();
//                //                auto& txq = _qs[i][engine()._id];
//                //                txq.flush_request_batch();
//                //                got += txq.process_completions();
//            }
//        }
//        got != 0
//    }
//
//    pub fn pure_poll_queues(&self, reactor: &Reactor) -> bool {
//        false
//    }
//
//    fn process_incoming(&self, reactor: &Reactor) {}
//
//    //    void start(unsigned cpuid);
//    //    template<size_t PrefetchCnt, typename Func>
//    //    size_t process_queue(lf_queue& q, Func process);
//    //    size_t process_incoming();
//    //    size_t process_completions();
//    //    private:
//    //    void work();
//    //    void submit_item(work_item* wi);
//    //    void respond(work_item* wi);
//    //    void move_pending();
//    //    void flush_request_batch();
//    //    void flush_response_batch();
//    //    bool has_unflushed_responses() const;
//    //    bool pure_poll_rx() const;
//    //    bool pure_poll_tx() const;
//}
//
//pub struct SmpPollFn<'a> {
//    smp_queues: &'a SmpQueues,
//    reactor: &'a Reactor,
//}
//
//impl<'a> PollFn for SmpPollFn<'a> {
//    fn poll(&self) -> bool {
//        self.smp_queues.poll_queues(self.reactor)
//    }
//
//    fn pure_poll(&self) -> bool {
//        self.smp_queues.pure_poll_queues(self.reactor)
//    }
//}
//
//impl<'a> SmpPollFn<'a> {
//    pub fn new<'b>(smp_queues: &'b SmpQueues, reactor: &'b Reactor) -> SmpPollFn<'b> {
//        SmpPollFn {
//            smp_queues: smp_queues,
//            reactor: reactor,
//        }
//    }
//}

