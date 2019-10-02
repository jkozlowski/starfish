use crate::shared::Shared;
use futures::channel::oneshot;
use futures::channel::oneshot::channel;
use futures::future::Shared as SharedFut;
use tokio::sync::watch;

/// Facility to stop new requests, and to tell when existing requests are done.
///
/// When stopping a service that serves asynchronous requests, we are faced with
/// two problems: preventing new requests from coming in, and knowing when existing
/// requests have completed.  The \c gate class provides a solution.
#[derive(Clone)]
pub struct Gate {
    inner: Shared<Inner>,
}

pub struct GateGuard {}

struct Inner {
    count: usize,
    stopped: Option<(oneshot::Sender<()>, SharedFut<oneshot::Receiver<()>>)>,
}

impl Gate {
    pub fn new() -> Self {
        Gate {
            inner: Shared::new(Inner {
                count: 0,
                stopped: None,
            }),
        }
    }

    // Registers an in-progress request.
    //
    // If the gate is not closed, the request is registered. Otherwise, error is returned.
}

//    Registers an in-progress request.
//    //
//    // If the gate is not closed, the request is registered.  Otherwise,
//    // a \ref gate_closed_exception is thrown.
//    void enter() {
//    if (_stopped) {
//    throw gate_closed_exception();
//    }
//    ++_count;
//    }
//    /// Unregisters an in-progress request.
//    ///
//    /// If the gate is closed, and there are no more in-progress requests,
//    /// the \ref closed() promise will be fulfilled.
//    void leave() {
//    --_count;
//    if (!_count && _stopped) {
//    _stopped->set_value();
//    }
//    }
//    /// Potentially stop an in-progress request.
//    ///
//    /// If the gate is already closed, a \ref gate_closed_exception is thrown.
//    /// By using \ref enter() and \ref leave(), the program can ensure that
//    /// no further requests are serviced. However, long-running requests may
//    /// continue to run. The check() method allows such a long operation to
//    /// voluntarily stop itself after the gate is closed, by making calls to
//    /// check() in appropriate places. check() with throw an exception and
//    /// bail out of the long-running code if the gate is closed.
//    void check() {
//    if (_stopped) {
//    throw gate_closed_exception();
//    }
//    }
//    /// Closes the gate.
//    ///
//    /// Future calls to \ref enter() will fail with an exception, and when
//    /// all current requests call \ref leave(), the returned future will be
//    /// made ready.
//    future<> close() {
//    assert(!_stopped && "seastar::gate::close() cannot be called more than once");
//    _stopped = compat::make_optional(promise<>());
//    if (!_count) {
//    _stopped->set_value();
//    }
//    return _stopped->get_future();
//    }
//
//    /// Returns a current number of registered in-progress requests.
//    size_t get_count() const {
//    return _count;
//    }
//
//    /// Returns whether the gate is closed.
//    bool is_closed() const {
//    return bool(_stopped);
//    }
//    };
//
//    /// Executes the function \c func making sure the gate \c g is properly entered
//    /// and later on, properly left.
//    ///
//    /// \param func function to be executed
//    /// \param g the gate. Caller must make sure that it outlives this function.
//    /// \returns whatever \c func returns
//    ///
//    /// \relates gate
//    template <typename Func>
//    inline
//    auto
//    with_gate(gate& g, Func&& func) {
//    g.enter();
//    return futurize_apply(std::forward<Func>(func)).finally([&g] { g.leave(); });
//    }
//    /// @}
