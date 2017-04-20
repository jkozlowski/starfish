use bounded_spsc_queue;
use bounded_spsc_queue::{Producer, Consumer};
use futures::Future;
use futures::unsync::oneshot::channel;
use futures::unsync::oneshot::Sender;
use futures::unsync::oneshot::Receiver;
use reactor;
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

pub struct SmpMessageQueueProducer {
    queue: Producer<Box<AsyncItem>>,
    remote: ReactorHandle,
}

impl SmpMessageQueueProducer {
    pub fn new(queue: Producer<Box<AsyncItem>>, remote: ReactorHandle) -> SmpMessageQueueProducer {
        SmpMessageQueueProducer {
            queue: queue,
            remote: remote,
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

        // TODO: This can block, so really we should implement this in the future,
        // TODO: so we can wait.
        self.queue.push(Box::new(async_message));
        //        let keep_running_flag = Arc::new(AtomicBool::new(false));
        //        // AssertUnwindSafe is used here becuase `Send + 'static` is basically
        //        // an alias for an implementation of the `UnwindSafe` trait but we can't
        //        // express that in the standard library right now.
        //        let sender = MySender {
        //            fut: AssertUnwindSafe(f).catch_unwind(),
        //            tx: Some(tx),
        //            keep_running_flag: keep_running_flag.clone(),
        //        };
        //        executor::spawn(sender).execute(self.inner.clone());
        //        CpuFuture { inner: rx , keep_running_flag: keep_running_flag.clone() }
        rcv
    }

    //    futurize_t<std::result_of_t<Func()>> submit(Func&& func) {
    //        auto wi = new async_work_item<Func>(std::forward<Func>(func));
    //        auto fut = wi->get_future();
    //        submit_item(wi);
    //        return fut;
    //    }
}

pub struct SmpMessageQueueConsumer {
    queue: Consumer<Box<AsyncItem>>,
    remote: ReactorHandle,
}

impl SmpMessageQueueConsumer {
    pub fn new(queue: Consumer<Box<AsyncItem>>, remote: ReactorHandle) -> SmpMessageQueueConsumer {
        SmpMessageQueueConsumer {
            queue: queue,
            remote: remote,
        }
    }
}

const QUEUE_LENGTH: usize = 128;

pub fn make_smp_message_queue(from: ReactorHandle,
                              to: ReactorHandle)
                              -> (SmpMessageQueueProducer, SmpMessageQueueConsumer) {
    let (p, c) = bounded_spsc_queue::make(QUEUE_LENGTH);
    let producer = SmpMessageQueueProducer::new(p, to);
    let consumer = SmpMessageQueueConsumer::new(c, from);
    (producer, consumer)
}

pub struct SmpQueues {
    producers: Vec<SmpMessageQueueProducer>,
    consumers: Vec<SmpMessageQueueConsumer>,
    reactor_id: usize,
    smp_count: usize,
}

impl SmpQueues {
    pub fn new(producers: Vec<SmpMessageQueueProducer>,
               consumers: Vec<SmpMessageQueueConsumer>,
               reactor_id: usize,
               smp_count: usize)
               -> SmpQueues {
        assert!(producers.len() == smp_count,
                "producers.len: expected {}, found {}",
                smp_count,
                producers.len());
        assert!(consumers.len() == smp_count,
                "consumers.len: expected {}, found {}",
                smp_count,
                consumers.len());
        SmpQueues {
            producers: producers,
            consumers: consumers,
            reactor_id: reactor_id,
            smp_count: smp_count,
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
            self.producers[reactor_id].submit(f)
        }
    }

    pub fn poll_queues(&self, reactor: &Reactor) -> bool {
        let mut got: usize = 0;
        for i in 0..self.smp_count {
            if reactor.id() != i {
                //                auto& rxq = _qs[engine().cpu_id()][i];
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

    fn process_incoming(&self, reactor: &Reactor) {
    }

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

//reactor::smp_pollfn::poll
//-> smp::poll_queues
//    -> smp_message_queue::flush_response_batch
//        * go through _completed_fifo and push them into _completed.
//    -> smp_message_queue::has_unflushed_responses
//        * check if _completed_fifo is not empty
//    -> smp_message_queue::process_incoming
//        * calls smp_message_queue::process_queue on _pending and installs a then future on each work item
//          that calls smp_message_queue::respond
//        -> smp_message_queue::process_queue
//            * uses prefetching to pop values from lf_queue (https://crates.io/crates/prefetch)
//        -> smp_message_queue::respond
//            * push the response onto _completed_fifo
//              and optionally smp_message_queue::flush_response_batch
//    -> smp_message_queue::flush_request_batch
//    -> smp_message_queue::process_completions
//
