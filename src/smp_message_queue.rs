use bounded_spsc_queue;
use bounded_spsc_queue::{Producer, Consumer};
use sys::imp::reactor_handle::ReactorHandle;

pub struct Message {
    // MySender<F, T>
}

pub struct SmpMessageQueueProducer {
    queue: Producer<Message>,
    remote: ReactorHandle,
}

impl SmpMessageQueueProducer {
    pub fn new(queue: Producer<Message>, remote: ReactorHandle) -> SmpMessageQueueProducer {
        SmpMessageQueueProducer {
            queue: queue,
            remote: remote,
        }
    }

    //    futurize_t<std::result_of_t<Func()>> submit(Func&& func) {
    //        auto wi = new async_work_item<Func>(std::forward<Func>(func));
    //        auto fut = wi->get_future();
    //        submit_item(wi);
    //        return fut;
    //    }
}

pub struct SmpMessageQueueConsumer {
    queue: Consumer<Message>,
    remote: ReactorHandle,
}

impl SmpMessageQueueConsumer {
    pub fn new(queue: Consumer<Message>, remote: ReactorHandle) -> SmpMessageQueueConsumer {
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
}

impl SmpQueues {
    pub fn new(producers: Vec<SmpMessageQueueProducer>,
               consumers: Vec<SmpMessageQueueConsumer>,
               reactor_id: usize)
               -> SmpQueues {
        SmpQueues {
            producers: producers,
            consumers: consumers,
            reactor_id: reactor_id,
        }
    }

    pub fn reactor_id(&self) -> usize {
        self.reactor_id
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
//struct async_work_item : work_item {
//        Func _func;
//        using futurator = futurize<std::result_of_t<Func()>>;
//        using future_type = typename futurator::type;
//        using value_type = typename future_type::value_type;
//        std::experimental::optional<value_type> _result;
//        std::exception_ptr _ex; // if !_result
//        typename futurator::promise_type _promise; // used on local side
//        async_work_item(Func&& func) : _func(std::move(func)) {}
//        virtual future<> process() override {
//            try {
//                return futurator::apply(this->_func).then_wrapped([this] (auto&& f) {
//                    try {
//                        _result = f.get();
//                    } catch (...) {
//                        _ex = std::current_exception();
//                    }
//                });
//            } catch (...) {
//                _ex = std::current_exception();
//                return make_ready_future();
//            }
//        }
//        virtual void complete() override {
//            if (_result) {
//                _promise.set_value(std::move(*_result));
//            } else {
//                // FIXME: _ex was allocated on another cpu
//                _promise.set_exception(std::move(_ex));
//            }
//        }
//        future_type get_future() { return _promise.get_future(); }
//    };
