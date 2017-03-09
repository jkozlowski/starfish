use bounded_spsc_queue;
use bounded_spsc_queue::{Producer, Consumer};
use reactor::Reactor;
use smp::UnsafePtr;

pub struct Message {
    val: usize
}

pub struct SmpMessageQueueProducer {
    queue: Producer<Message>,
    remote: UnsafePtr<Reactor>
}

impl SmpMessageQueueProducer {
    pub fn new(queue: Producer<Message>, remote: UnsafePtr<Reactor>) -> SmpMessageQueueProducer {
        SmpMessageQueueProducer {
            queue: queue,
            remote: remote
        }
    }
}

pub struct SmpMessageQueueConsumer {
    queue: Consumer<Message>,
    remote: UnsafePtr<Reactor>
}

impl SmpMessageQueueConsumer {
    pub fn new(queue: Consumer<Message>, remote: UnsafePtr<Reactor>) -> SmpMessageQueueConsumer {
        SmpMessageQueueConsumer {
            queue: queue,
            remote: remote
        }
    }
}

const QUEUE_LENGTH: usize = 128;

pub fn make_smp_message_queue(from: UnsafePtr<Reactor>, to: UnsafePtr<Reactor>) -> (SmpMessageQueueProducer, SmpMessageQueueConsumer) {
    let (p, c) = bounded_spsc_queue::make(QUEUE_LENGTH);
    let producer = SmpMessageQueueProducer::new(p, to);
    let consumer = SmpMessageQueueConsumer::new(c, from);
    (producer, consumer)
}

pub struct SmpQueues {
    producers: Vec<SmpMessageQueueProducer>,
    consumers: Vec<SmpMessageQueueConsumer>,
    reactor_id: usize
}

impl SmpQueues {
    pub fn new(producers: Vec<SmpMessageQueueProducer>,
               consumers: Vec<SmpMessageQueueConsumer>,
               reactor_id: usize) -> SmpQueues {
        SmpQueues {
            producers: producers,
            consumers: consumers,
            reactor_id: reactor_id
        }
    }

    pub fn reactor_id(&self) -> usize {
        self.reactor_id
    }
}
