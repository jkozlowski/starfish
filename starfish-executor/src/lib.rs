#![feature(pin, arbitrary_self_types, futures_api)]

#[macro_use]
extern crate futures;

use futures::future::FutureObj;
use futures::future::{Future, LocalFutureObj};
use futures::task::{Context, Executor, LocalWaker, Poll, UnsafeWake, Waker};
use futures::task::{SpawnErrorKind, SpawnObjError};
use std::boxed::PinBox;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::mem::PinMut;
use std::ptr::NonNull;
use std::rc::Rc;

/// Tracks the executor for the current execution context.
/// TODO: Use UnsafeCell, since we can guarantee correct implementation
thread_local!(static CURRENT_EXECUTOR: RefCell<Option<CurrentThreadExecutor>> = RefCell::new(None));

struct LocalThreadExecutor {}

impl LocalThreadExecutor {
    fn new() -> LocalThreadExecutor {
        LocalThreadExecutor {}
    }
}

impl Executor for LocalThreadExecutor {
    fn spawn_obj(&mut self, task: FutureObj<'static, ()>) -> Result<(), SpawnObjError> {
        Err(SpawnObjError {
            task,
            kind: SpawnErrorKind::shutdown(),
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

// TODO: Revisit this, even though we only ever want to run local futures,
// however I'll need to figure out how to have local counters that send a message once the value reaches zero
// to the original core.
//
// It is likely that CurrentThreadExecutor will need to store it's ID and a function that will let it notify
// other cores (that just maps to dpdk queues); Then TaskHandle also needs to know which core it is on
// and then UnsafeWake impls will actually do nested Rcs -> there is basically a counter on each core
// that has access and if a particular core's handle goes to 0, it notifies the core that sent it the handle,
// so they form a cycle?
//
// Not sure if this is even possible, so for now let's pretend we'll never send handles to other cores.
unsafe impl Send for TaskHandle {}
unsafe impl Sync for TaskHandle {}

unsafe impl UnsafeWake for TaskHandle {
    unsafe fn clone_raw(&self) -> Waker {
        Waker::new(clone_task_handle_ptr(self))
    }

    unsafe fn drop_raw(&self) {
        // This will drop the Rc
        let _ = Rc::from_raw(self);
    }

    unsafe fn wake(&self) {
        if !self.queued.replace(true) {
            CURRENT_EXECUTOR.with(|current| match *current.borrow() {
                Some(ref current_thread) => {
                    let self_clone = clone_task_handle(self);
                    current_thread.tq.borrow_mut().add_task(self_clone);
                }
                _ => { /* Executor is gone :( */ }
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
            fut: PinBox::new(future).into(),
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
            _ => return Enter {},
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
                    // We need to increase the counter temporarily,
                    // so that the waker does not drop the task still owned by Bomb
                    // when it drops;
                    // TODO: probably can do away with this clone.
                    let clone_task_handle_ptr =
                        Rc::into_raw(bomb.task_handle.as_ref().unwrap().clone());
                    // TODO: I think I should be able to use some of the functions from below
                    let waker = LocalWaker::new(NonNull::new_unchecked(
                        clone_task_handle_ptr as *mut TaskHandle,
                    ));
                    // TODO: need the executor bit done
                    let mut executor = LocalThreadExecutor::new();
                    let mut cx = Context::new(&waker, &mut executor);
                    let future = PinMut::new_unchecked(&mut task.fut);
                    future.poll(&mut cx)
                };

                match res {
                    Poll::Pending => {
                        // We have transferred the reference to the task to Waker
                        // So need to move it out of Bomb and put back the task
                        let task_handle = bomb.task_handle.take().unwrap();

                        *task_handle.task.get() = Some(task);

                        // TODO: Figure out if I need to drop the task handle
                        continue;
                    }
                    _ => {}
                };
            }
        }
    })
}

fn with_default<F, R>(f: F) -> R
where
    F: FnOnce(&CurrentThreadExecutor) -> R,
{
    CURRENT_EXECUTOR.with(|current| match *current.borrow() {
        Some(ref current_thread) => {
            return f(current_thread);
        }
        None => panic!("Executor not set"),
    })
}

fn with_queue<F, R>(f: F) -> R
where
    F: FnOnce(&mut TaskQueue) -> R,
{
    with_default(|executor| f(&mut executor.tq.borrow_mut()))
}

fn clone_task_handle(task_handle: &TaskHandle) -> Rc<TaskHandle> {
    let self_as_rc = unsafe { Rc::from_raw(task_handle) };
    let self_clone = self_as_rc.clone();

    // We need to make sure self_as_rc does not drop,
    // since it is STILL referenced by this TaskHandle
    forget_rc(self_as_rc);

    return self_clone;
}

fn clone_task_handle_ptr(task_handle: &TaskHandle) -> NonNull<TaskHandle> {
    // TODO: should be fine right?
    unsafe {
        return NonNull::new_unchecked(
            Rc::into_raw(clone_task_handle(task_handle)) as *mut TaskHandle
        );
    }
}

fn forget_rc(task_handle: Rc<TaskHandle>) {
    let _ = Rc::into_raw(task_handle);
}

#[cfg(test)]
#[macro_use]
extern crate hamcrest;

#[cfg(test)]
#[macro_use]
extern crate futures_core as futures_core_macros;

#[cfg(test)]
mod tests {

    use super::*;
    use futures_core::task;
    use futures_core::task::Poll;
    use hamcrest::prelude::*;

    struct Controller {
        pollers: VecDeque<Box<Fn(&mut task::Context, &mut Controller) -> Poll<()>>>,
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

        fn save_waker(&mut self, waker: Waker) {
            match self.waker {
                Some(_) => panic!("Waker already saved"),
                None => self.waker = Some(waker),
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
            P: Fn(&mut task::Context, &mut Controller) -> Poll<()> + 'static,
        {
            self.pollers.push_back(Box::new(poller))
        }

        fn pop_pollers(
            &mut self,
        ) -> Option<Box<Fn(&mut task::Context, &mut Controller) -> Poll<()>>> {
            self.pollers.pop_front()
        }
    }

    struct MockFuture {
        controller: Rc<RefCell<Controller>>,
    }

    impl MockFuture {
        fn new(controller: Rc<RefCell<Controller>>) -> MockFuture {
            MockFuture {
                controller: controller,
            }
        }
    }

    impl Drop for MockFuture {
        fn drop(&mut self) {
            self.controller.borrow_mut().dropped();
        }
    }

    impl Future for MockFuture {
        type Output = ();

        fn poll(self: PinMut<Self>, ctx: &mut task::Context) -> Poll<()> {
            let mut ctrl = self.controller.borrow_mut();
            let poller = match ctrl.pop_pollers() {
                Some(poller) => poller,
                None => panic!("Called poll when not expected"),
            };
            ctrl.poll();
            return poller(ctx, &mut ctrl);
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
            ctrl.borrow_mut().push_pollers(|ctx, ctrl| {
                ctrl.save_waker(ctx.waker().clone());
                Poll::Pending
            });

            assert_pure_poll(&ctrl, 1, false);

            // Notify future multiple times
            let waker = ctrl.borrow_mut().unwrap_waker();
            waker.wake();
            waker.wake();

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
            ctrl.borrow_mut().push_pollers(|ctx, _| {
                // Wake multiple times
                ctx.waker().wake();
                ctx.waker().wake();

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
        spawn(MockFuture::new(ctrl.clone()));

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
