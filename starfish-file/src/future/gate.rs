use crate::shared::Shared;
use futures::channel::oneshot;
use futures::channel::oneshot::channel;
use futures::future::Shared as SharedFut;
use tokio::sync::watch;
use std::fmt;
use std::error;
use std::mem;

/// Facility to stop new requests, and to tell when existing requests are done.
///
/// When stopping a service that serves asynchronous requests, we are faced with
/// two problems: preventing new requests from coming in, and knowing when existing
/// requests have completed.  The \c gate class provides a solution.
#[derive(Clone)]
pub struct Gate {
    inner: Shared<Inner>,
}

pub struct GateGuard {
    inner: Shared<Inner>,
}

struct Inner {
    count: usize,
    closed: Option<oneshot::Sender<()>>,
}

impl Drop for GateGuard {
    fn drop(&mut self) {
        self.inner.borrow_mut().count -= 1;
        if self.inner.count != 0 {
            return;
        }
        if self.inner.closed.is_some()
        {
            let sender = {
                let inner = &mut self.inner.borrow_mut().closed;
                mem::replace(inner, None).unwrap()
            };
            sender.send(()).unwrap();
        }
    }
}

#[derive(Debug)]
pub struct Closed;

impl fmt::Display for Closed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gate closed")
    }
}

impl std::error::Error for Closed {}

impl Gate {
    pub fn new() -> Self {
        Gate {
            inner: Shared::new(Inner {
                count: 0,
                closed: None,
            }),
        }
    }

    /// Registers an in-progress request.
    ///
    /// If the gate is not closed, the request is registered. Otherwise, error is returned.
    pub fn enter(&self) -> Result<GateGuard, Closed> {
        if self.inner.closed.is_some() {
            return Err(Closed);
        }
        self.inner.borrow_mut().count += 1;
        Ok(GateGuard {
            inner: self.inner.clone()
        })
    }

    /// Potentially stop an in-progress request.
    ///
    /// If the gate is already closed, error is returned.
    /// By using ``enter``, the program can ensure that
    /// no further requests are serviced. However, long-running requests may
    /// continue to run. The ``check()`` method allows such a long operation to
    /// voluntarily stop itself after the gate is closed, by making calls to
    /// ``check()`` in appropriate places. ``check()`` will return error and
    /// bail out of the long-running code if the gate is closed.
    pub fn check(&self) -> Result<(), Closed> {
        if self.inner.closed.is_some() {
            return Err(Closed);
        }
        Ok(())
    }

    /// Closes the gate.
    ///
    /// Future calls to ``enter()`` will error, and when
    /// all current requests finish, the returned future will be
    /// made ready.
    pub async fn close(&self) {
        // TODO: Throwing here kinda sucks, should just save the received and await
        assert!(self.inner.closed.is_none(), "close() cannot be called more than once");
        if self.inner.count == 0 {
            return;
        }
        let (sender, receiver) = oneshot::channel();
        self.inner.borrow_mut().closed = Some(sender);
        receiver.await.unwrap();
    }

    /// Returns a current number of registered in-progress requests.
    pub fn get_count(&self) -> usize {
        self.inner.count
    }

    /// Returns whether the gate is closed.
    pub fn is_closed(&self) -> bool {
        return self.inner.closed.is_some();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
