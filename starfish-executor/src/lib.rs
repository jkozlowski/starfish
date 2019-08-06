#![warn(rust_2018_idioms)]
#![feature(arbitrary_self_types, nll)]

extern crate futures;

macro_rules! waker_vtable {
    ($ty:ident) => {
        &RawWakerVTable::new(
            clone_rc_raw::<$ty>,
            wake_rc_raw::<$ty>,
            wake_by_ref_rc_raw::<$ty>,
            drop_rc_raw::<$ty>,
        )
    };
}

pub mod waker;
pub mod waker_ref;

use crate::waker::RcWake;
use futures::future::LocalFutureObj;
use futures::task::LocalSpawn;
use futures::task::SpawnError;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;

// Tracks the executor for the current execution context.
// TODO: Use UnsafeCell, since we can guarantee correct implementation
thread_local!(static CURRENT_EXECUTOR: RefCell<Option<CurrentThreadExecutor>> = RefCell::new(None));

#[allow(dead_code)]
struct LocalExecutor {}

impl LocalExecutor {
    #[allow(dead_code)]
    fn new() -> Self {
        LocalExecutor {}
    }
}

impl LocalSpawn for LocalExecutor {
    fn spawn_local_obj(&mut self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        with_default_no_fail(|maybe_executor| match maybe_executor {
            Some(ref executor) => {
                let task = Rc::new(TaskHandle {
                    task: UnsafeCell::new(Some(TaskContext::new(future))),
                    queued: Cell::new(true),
                });
                (&mut executor.tq.borrow_mut()).add_task(task);
                Ok(())
            }
            None => Err(SpawnError::shutdown()),
        })
    }
}

pub struct CurrentThreadExecutor {
    // Mutable interior is required, in case a future
    // asks to be woken up while we are in `pure_poll`.
    // TODO: use UnsafeCell, since we know the usage is correct and apparently there is some overhead.
    tq: RefCell<TaskQueue>,
}

impl CurrentThreadExecutor {
    fn new() -> CurrentThreadExecutor {
        CurrentThreadExecutor {
            tq: RefCell::new(TaskQueue::new()),
        }
    }
}

struct TaskQueue {
    q: VecDeque<Rc<TaskHandle>>,
}

impl TaskQueue {
    #[inline]
    fn new() -> TaskQueue {
        TaskQueue { q: VecDeque::new() }
    }

    #[inline]
    fn add_task(&mut self, task: Rc<TaskHandle>) {
        self.q.push_back(task)
    }

    /// Polls the next `TaskHandle` and gets it as a raw pointer from `Rc`.
    /// The counter is not incremented, the pointer owns one reference.
    #[inline]
    fn poll_task_from_rc(&mut self) -> Option<*const TaskHandle> {
        self.q.pop_front().map(Rc::into_raw)
    }
}

struct TaskHandle {
    task: UnsafeCell<Option<TaskContext>>,
    queued: Cell<bool>,
}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        unsafe {
            if (*self.task.get()).is_some() {
                // TODO(jkozlowski) Fix this up.
                //abort("future still here when dropping");
            }
        }
    }
}

// TODO: Revisit this, even though we only ever want to run local futures,
// however I'll need to figure out how to have local counters that
// send a message once the value reaches zero
// to the original core.
//
// It is likely that CurrentThreadExecutor will need to store it's ID and
// a function that will let it notify
// other cores (that just maps to dpdk queues); Then TaskHandle also needs
// to know which core it is on
// and then UnsafeWake impls will actually do nested Rcs -> there is basically
// a counter on each core
// that has access and if a particular core's handle goes to 0, it notifies
// the core that sent it the handle,
// so they form a cycle?
//
// Not sure if this is even possible, so for now let's pretend
// we'll never send handles to other cores.
unsafe impl Send for TaskHandle {}
unsafe impl Sync for TaskHandle {}

impl RcWake for TaskHandle {
    fn wake_by_ref(rc_self: &Rc<Self>) {
        if !rc_self.queued.replace(true) {
            CURRENT_EXECUTOR.with(|current| {
                if let Some(ref current_thread) = *current.borrow() {
                    // Note that we don't change the reference count of the task here,
                    // we merely enqueue the raw pointer. The `pure_poll`
                    // implementation guarantees that if we set the `queued` flag that
                    // there's a reference count held by the main `pure_poll` queue
                    // still.
                    // TODO(jkozlowski) Fix this clone
                    current_thread.tq.borrow_mut().add_task(rc_self.clone());
                }
            });
        }
    }
}

struct TaskContext {
    fut: LocalFutureObj<'static, ()>,
}

impl TaskContext {
    fn new<F>(future: F) -> TaskContext
    where
        F: Future<Output = ()> + 'static,
    {
        TaskContext {
            fut: Box::new(future).into(),
        }
    }
}

pub struct Enter {}

impl Drop for Enter {
    fn drop(&mut self) {
        CURRENT_EXECUTOR.with(|current| {
            if current.borrow().as_ref().is_none() {
                panic!("Executor not initialized")
            }

            match current.replace(None) {
                Some(_) => { /* Executor is dropped here nicely */ }
                _ => panic!("Executor already initialized"),
            }
        })
    }
}

pub fn initialize() -> Enter {
    CURRENT_EXECUTOR.with(|current| {
        if current.borrow().as_ref().is_some() {
            panic!("Executor already initialized");
        }

        let executor = CurrentThreadExecutor::new();

        match current.replace(Some(executor)) {
            Some(_) => panic!("Executor already initialized"),
            _ => Enter {},
        }
    })
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    let task = Rc::new(TaskHandle {
        task: UnsafeCell::new(Some(TaskContext::new(future))),
        queued: Cell::new(true),
    });

    with_queue(|queue| queue.add_task(task))
}

pub fn pure_poll() -> bool {
    with_default(|executor| {
        let mut ret = false;
        loop {
            let task_handle_ptr = match executor.tq.borrow_mut().poll_task_from_rc() {
                Some(rc_task_handle) => rc_task_handle,
                None => {
                    return ret;
                }
            };

            unsafe {
                let mut task = match (*(*task_handle_ptr).task.get()).take() {
                    Some(task) => task,
                    None => {
                        // The future has gone away; just need to make sure
                        // we invoke Drop on task_handle_ptr
                        let _node = Rc::from_raw(task_handle_ptr);
                        continue;
                    }
                };

                // Unset queued flag... this must be done before
                // polling. This ensures that the future gets
                // rescheduled if it is notified **during** a call
                // to `pure_poll`.
                let prev = (*task_handle_ptr).queued.replace(false);
                assert!(prev);

                ret = true;

                struct Bomb {
                    task_handle: Option<Rc<TaskHandle>>,
                }

                // Bomb now owns task_handle_ptr
                let mut bomb = Bomb {
                    task_handle: Some(Rc::from_raw(task_handle_ptr)),
                };

                let res = {
                    let waker = waker_ref::waker_ref(bomb.task_handle.as_ref().unwrap());
                    let mut cx = Context::from_waker(&waker);

                    let future = Pin::new_unchecked(&mut task.fut);
                    future.poll(&mut cx)
                };

                if let Poll::Pending = res {
                    // We have transferred the reference to the task to Waker
                    // So need to move it out of Bomb and put back the task
                    let task_handle = bomb.task_handle.take().unwrap();

                    *task_handle.task.get() = Some(task);

                    // TODO: Figure out if I need to drop the task handle
                    continue;
                }
            }
        }
    })
}

fn with_default<F, R>(f: F) -> R
where
    F: FnOnce(&CurrentThreadExecutor) -> R,
{
    CURRENT_EXECUTOR.with(|current| match *current.borrow() {
        Some(ref current_thread) => f(current_thread),
        None => panic!("Executor not set"),
    })
}

fn with_default_no_fail<F, E, R>(f: F) -> Result<R, E>
where
    F: FnOnce(Option<&CurrentThreadExecutor>) -> Result<R, E>,
{
    CURRENT_EXECUTOR.with(|current| match *current.borrow() {
        Some(ref current_thread) => f(Some(current_thread)),
        None => f(None),
    })
}

fn with_queue<F, R>(f: F) -> R
where
    F: FnOnce(&mut TaskQueue) -> R,
{
    with_default(|executor| f(&mut executor.tq.borrow_mut()))
}

pub fn abort(s: &str) -> ! {
    struct DoublePanic;

    impl Drop for DoublePanic {
        fn drop(&mut self) {
            //panic!("panicking twice to abort the program");
        }
    }

    let _bomb = DoublePanic;
    panic!("{}", s);
}

#[cfg(test)]
#[macro_use]
extern crate hamcrest2;

#[cfg(test)]
mod tests {

    use super::*;
    use futures::task::Poll;
    use hamcrest2::prelude::*;
    use std::task::Waker;

    type PollerFn = dyn Fn(&Waker, &mut Controller) -> Poll<()>;

    struct Controller {
        pollers: VecDeque<Box<PollerFn>>,
        poll_count: usize,
        dropped: bool,
        waker: Option<Waker>,
    }

    impl Controller {
        fn new() -> Controller {
            Controller {
                poll_count: 0,
                dropped: false,
                pollers: VecDeque::new(),
                waker: None,
            }
        }

        fn save_waker(&mut self, waker: &Waker) {
            match self.waker {
                Some(_) => panic!("Waker already saved"),
                None => self.waker = Some(waker.clone()),
            }
        }

        fn unwrap_waker(&mut self) -> Waker {
            self.waker.take().unwrap()
        }

        fn dropped(&mut self) {
            assert!(!self.dropped, "Already dropped");
            self.dropped = true;
        }

        fn is_dropped(&self) -> bool {
            self.dropped
        }

        fn poll(&mut self) {
            self.poll_count += 1;
        }

        fn poll_count(&self) -> usize {
            self.poll_count
        }

        fn push_pollers<P>(&mut self, poller: P)
        where
            P: Fn(&Waker, &mut Controller) -> Poll<()> + 'static,
        {
            self.pollers.push_back(Box::new(poller))
        }

        fn pop_pollers(&mut self) -> Option<Box<PollerFn>> {
            self.pollers.pop_front()
        }
    }

    struct MockFuture {
        controller: Rc<RefCell<Controller>>,
    }

    impl MockFuture {
        fn new(controller: Rc<RefCell<Controller>>) -> Self {
            MockFuture { controller }
        }
    }

    impl Drop for MockFuture {
        fn drop(&mut self) {
            self.controller.borrow_mut().dropped();
        }
    }

    impl Future for MockFuture {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            let mut ctrl = self.controller.borrow_mut();
            let poller = match ctrl.pop_pollers() {
                Some(poller) => poller,
                None => panic!("Called poll when not expected"),
            };
            ctrl.poll();
            poller(cx.waker(), &mut ctrl)
        }
    }

    #[test]
    fn if_waker_is_not_saved_future_is_dropped() {
        mock_test(|ctrl| {
            // Return pending but do not keep a Waker reference around
            ctrl.borrow_mut().push_pollers(|_, _| Poll::Pending);

            assert_pure_poll(&ctrl, 1, true);
        })
    }

    #[test]
    fn future_is_notified_from_outside_poll() {
        mock_test(|ctrl| {
            // Save waker
            ctrl.borrow_mut().push_pollers(|lw, ctrl| {
                ctrl.save_waker(lw);
                Poll::Pending
            });

            assert_pure_poll(&ctrl, 1, false);

            // Notify future multiple times
            let waker = ctrl.borrow_mut().unwrap_waker();
            waker.wake_by_ref();
            waker.wake_by_ref();

            // Do not save the waker this time
            ctrl.borrow_mut().push_pollers(|_, _| Poll::Pending);

            assert_pure_poll(&ctrl, 2, false);

            drop(waker);

            assert_dropped(&ctrl, true);
        })
    }

    #[test]
    fn future_is_polled_again_if_notified_from_poll() {
        mock_test(|ctrl| {
            ctrl.borrow_mut().push_pollers(|lw, _| {
                // Wake multiple times
                lw.wake_by_ref();
                lw.wake_by_ref();

                Poll::Pending
            });

            ctrl.borrow_mut().push_pollers(|_, _| Poll::Pending);

            assert_pure_poll(&ctrl, 2, true);
        })
    }

    // TODO: Spawn a future that spawns from poll.

    fn mock_test<F>(f: F)
    where
        F: Fn(Rc<RefCell<Controller>>),
    {
        let _enter = initialize();

        let ctrl = Rc::new(RefCell::new(Controller::new()));
        assert_that!(
            LocalExecutor::new().spawn_local_obj(Box::new(MockFuture::new(ctrl.clone())).into()),
            is(ok())
        );

        f(ctrl)
    }

    fn assert_pure_poll(ctrl: &Rc<RefCell<Controller>>, poll_count: usize, is_dropped: bool) {
        pure_poll();

        assert_that!(ctrl.borrow().poll_count(), is(equal_to(poll_count)));
        assert_dropped(ctrl, is_dropped);
    }

    fn assert_dropped(ctrl: &Rc<RefCell<Controller>>, is_dropped: bool) {
        assert_that!(ctrl.borrow().is_dropped(), is(is_dropped));
    }
}
