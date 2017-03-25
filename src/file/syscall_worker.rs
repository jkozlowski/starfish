////! A worker that blocks on syscalls
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;

use bounded_spsc_queue::{Producer, Consumer, make};
use futures::{IntoFuture, Future, Poll, Async};
use futures::future::lazy;
use futures::sync::oneshot::{channel, Sender, Receiver};
use futures::executor::{self, Run, Executor};

//// TODO:
// # Implement batched drain of the consumer queue
// # Allow for naming the worker thread
// # How to wakeup the main thread?
// # Also need, queue_has_room semaphore on the producer side
// # Signalling the main thread when it sleeps
// # Signalling the worker thread when the queue has something in it

////  static constexpr size_t queue_length = 128;
const QUEUE_LENGTH: usize = 128;

////  static constexpr size_t batch_size = 16;
const BATCH_SIZE: usize = 16;
////  static constexpr size_t prefetch_cnt = 2;
const PREFETCH_CNT: usize = 2;

pub struct SyscallWorker {
    producer: Producer<Message>,
}

//struct MySender<F, T> {
//    fut: F,
//    tx: Sender<T>,
//    keep_running_flag: Arc<AtomicBool>,
//}
//
//fn _assert() {
//    fn _assert_send<T: Send>() {}
//    fn _assert_sync<T: Sync>() {}
//    _assert_send::<Worker>();
//    _assert_sync::<Worker>();
//}
//
//struct Inner {}
//

enum Message {
    Run(Run),
    Close,
}

impl SyscallWorker {
    /// Create new worker
    pub fn new() -> SyscallWorker {
        let (producer, consumer) = make(QUEUE_LENGTH);
        let worker = SyscallWorker { producer: producer };

        thread::spawn(move || work(consumer));

        return worker;
    }

    //    pub fn spawn<F, R>(&self, f: F) -> WorkerFuture<R, ()>
    //        where F: FnOnce() -> R
    //    {
    //        let (tx, rx) = channel();
    //        let keep_running_flag = Arc::new(AtomicBool::new(false));
    //
    //        // AssertUnwindSafe is used here becuase `Send + 'static` is basically
    //        // an alias for an implementation of the `UnwindSafe` trait but we can't
    //        // express that in the standard library right now.
    ////        let sender = MySender {
    ////            fut: AssertUnwindSafe(f).catch_unwind(),
    ////            tx: Some(tx),
    ////            keep_running_flag: keep_running_flag.clone(),
    ////        };
    ////        executor::spawn(sender).execute();
    ////        WorkerFuture { inner: rx, keep_running_flag: keep_running_flag.clone() }
    //        unimplemented!();
    //    }
}

fn work(consumer: Consumer<Message>) {
    loop {
        match consumer.pop() {
            Message::Run(r) => r.run(),
            Message::Close => break,
        }
    }
}

//impl Drop for Worker {
//    fn drop(&mut self) {
//        if self.inner.cnt.fetch_sub(1, Ordering::Relaxed) == 1 {
//            for _ in 0..self.inner.size {
//                self.inner.queue.push(Message::Close);
//            }
//        }
//    }
//}

//impl Executor for Worker {
//    fn execute(&self, run: Run) {
//        self.queue.push(Message::Run(run))
//    }
//}
//

#[must_use]
pub struct WorkerFuture<T, E> {
    inner: Receiver<thread::Result<Result<T, E>>>,
    keep_running_flag: Arc<AtomicBool>,
}
//impl<T, E> WorkerFuture<T, E> {
//    /// Drop this future without canceling the underlying future.
//    ///
//    /// When `WorkerFuture` is dropped, `CpuPool` will try to abort the underlying
//    /// future. This function can be used when user wants to drop but keep
//    /// executing the underlying future.
//    pub fn forget(self) {
//        self.keep_running_flag.store(true, Ordering::SeqCst);
//    }
//}
//
//impl<T: Send + 'static, E: Send + 'static> Future for WorkerFuture<T, E> {
//    type Item = T;
//    type Error = E;
//
//    fn poll(&mut self) -> Poll<T, E> {
//        match self.inner.poll().expect("shouldn't be canceled") {
//            Async::Ready(Ok(Ok(e))) => Ok(e.into()),
//            Async::Ready(Ok(Err(e))) => Err(e),
//            Async::Ready(Err(e)) => panic::resume_unwind(e),
//            Async::NotReady => Ok(Async::NotReady),
//        }
//    }
//}
//
//impl<F: Future> Future for MySender<F, Result<F::Item, F::Error>> {
//    type Item = ();
//    type Error = ();
//
//    fn poll(&mut self) -> Poll<(), ()> {
//        if let Ok(Async::Ready(_)) = self.tx.as_mut().unwrap().poll_cancel() {
//            if !self.keep_running_flag.load(Ordering::SeqCst) {
//                // Cancelled, bail out
//                return Ok(().into())
//            }
//        }
//
//        let res = match self.fut.poll() {
//            Ok(Async::Ready(e)) => Ok(e),
//            Ok(Async::NotReady) => return Ok(Async::NotReady),
//            Err(e) => Err(e),
//        };
//        self.tx.take().unwrap().complete(res);
//        Ok(Async::Ready(()))
//    }
//}
