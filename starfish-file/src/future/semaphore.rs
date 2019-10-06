use std::fmt;

pub struct Semaphore {}

pub struct SemaphoreGuard {}

impl Drop for SemaphoreGuard {
    fn drop(&mut self) {}
}

#[derive(Debug)]
pub struct Broken;

impl fmt::Display for Broken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Semaphore broken")
    }
}

impl std::error::Error for Broken {}

// TODO(jkozlowski): How do timeouts/cancellations work here? I guess Drop will be invoked,
// so stuff needs to be drop happy, which it probably is naturally: like it doesn't make sense
// to increment any counters until there are any available. Similarly, if SemaphoreGuard is dropped,
// it is the same as if the request completed successfully.

/// Counted resource guard.
///
/// This is a standard computer science semaphore, adapted
/// for futures.  You can deposit units into a counter,
/// or take them away. Taking units from the counter may wait
/// if not enough units are available.
///
/// To support exceptional conditions, a ``broken()`` method
/// is provided, which causes all current waiters to stop waiting,
/// with an error future returned. This allows causing all
/// futures that are blocked on a semaphore to continue. This is
/// similar to POSIX's ``pthread_cancel()``, with ``wait()`` acting
/// as a cancellation point.
impl Semaphore {
    pub fn new(num_permits: usize) -> Self {
        Semaphore {}
    }

    /// Waits until at least a specific number of units are available in the
    /// counter, and reduces the counter by that amount of units.
    ///
    /// ``Waits are serviced in FIFO order, though if several are awakened at once, they may be reordered by the scheduler.``
    ///
    /// \param nr Amount of units to wait for (default 1).
    /// \return a future that becomes ready when sufficient units are available
    ///         to satisfy the request.  If the semaphore was \ref broken(), may
    ///         contain an exception.
    // TODO(jkozlowski): Actually implement, instead of lying
    pub async fn wait(&self, nr: usize) -> Result<SemaphoreGuard, Broken> {
        Ok(SemaphoreGuard {})
    }

    pub const fn may_proceed(&self, nr: usize) -> bool {
        //return has_available_units(nr) && _wait_list.empty();
        true
    }
}
