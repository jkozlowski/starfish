///// Simple library for signalling between threads.
///// Parent is able to signal the child through eventfd
///// and child through pthread_kill.
///// http://zorksylar.github.io/epoll-eventfd.html
///// https://www.microsoft.com/en-us/research/publication/gram-scaling-graph-computation-to-the-trillions/
///// https://github.com/wrl/thread-sync-latency-tests
//
//use eventfd::EventfdFd;
//
//use std::io;
//use std::sync::Arc;
//use std::thread::Builder;
//use std::thread::JoinHandle;
//
//pub struct ParentHandle<T> {
//    handle: JoinHandle<T>
//}
//
//impl ParentHandle {
//
//}
//
//pub struct ChildHandle {
//    eventfd: Arc<EventfdFd>
//}
//
//impl ChildHandle {
//    fn is_parent_stopped() -> bool {
//        // Check an atomic bool
//        true
//    }
//
//    fn maybe_wakeup() {
//        // This is how the threadpool->main thread communication happens
//        // For smp stuff, it's just signals.
//        //     std::atomic_signal_fence(std::memory_order_seq_cst);
//        //        if (remote->_sleeping.load(std::memory_order_relaxed)) {
//        //        // We are free to clear it, because we're sending a signal now
//        //        remote->_sleeping.store(false, std::memory_order_relaxed);
//        //        remote->wakeup();
//        //        }
//        //        if (_main_thread_idle.load(std::memory_order_seq_cst)) {
//        //            pthread_kill(_notify, SIGUSR1);
//        //        }
//    }
//}
//
//pub fn spawn<F, T>(builder: Builder, f: F) -> io::Result<ParentHandle<T>> where
//    F: FnOnce(ChildHandle) -> T, F: Send + 'static, T: Send + 'static
//{
//    let eventfd = Arc::new(EventfdFd::parent_to_child());
//    let child_handle = ChildHandle {};
//    Ok(ParentHandle {
//        handle: builder.spawn(move || f(child_handle))?
//    })
//}
//
//#[cfg(test)]
//mod tests {
//    #[test]
//    fn it_works() {
//
//    }
//}