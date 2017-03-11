use libc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Clone)]
pub struct ReactorHandle {
    sleeping: Arc<AtomicBool>,
    thread: libc::pthread_t
}

impl ReactorHandle {
    pub fn new(sleeping: Arc<AtomicBool>) -> ReactorHandle {
        ReactorHandle {
            sleeping: sleeping,
            // TODO: should check return value?
            thread: unsafe { libc::pthread_self() }
        }
    }

    pub fn maybe_wakeup() {
        //        // Called after lf_queue_base::push().
        //        //
        //        // This is read-after-write, which wants memory_order_seq_cst,
        //        // but we insert that barrier using systemwide_memory_barrier()
        //        // because seq_cst is so expensive.
        //        //
        //        // However, we do need a compiler barrier:
        //        std::atomic_signal_fence(std::memory_order_seq_cst);
        //        if (remote->_sleeping.load(std::memory_order_relaxed)) {
        //        // We are free to clear it, because we're sending a signal now
        //        remote->_sleeping.store(false, std::memory_order_relaxed);
        //        remote->wakeup();
        //        }
    }
}
