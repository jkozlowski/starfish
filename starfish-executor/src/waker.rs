use std::mem;
use std::rc::Rc;
use std::task::RawWaker;
use std::task::RawWakerVTable;
use std::task::Waker;

pub fn waker<W>(wake: Rc<W>) -> Waker
where
    W: RcWake,
{
    let ptr = Rc::into_raw(wake) as *const ();

    unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable!(W))) }
}

pub trait RcWake {
    fn wake(self: Rc<Self>) {
        Self::wake_by_ref(&self)
    }

    fn wake_by_ref(rc_self: &Rc<Self>);
}

// used by `waker_ref`; impl for `RcWake`
pub(super) unsafe fn clone_rc_raw<T: RcWake>(data: *const ()) -> RawWaker {
    increase_refcount::<T>(data);
    RawWaker::new(data, waker_vtable!(T))
}

// impl for `RcWake`
unsafe fn wake_rc_raw<T: RcWake>(data: *const ()) {
    let arc: Rc<T> = Rc::from_raw(data as *const T);
    RcWake::wake(arc);
}

// used by `waker_ref`; impl for `RcWake`
pub(super) unsafe fn wake_by_ref_rc_raw<T: RcWake>(data: *const ()) {
    let arc: Rc<T> = Rc::from_raw(data as *const T);
    RcWake::wake_by_ref(&arc);
    mem::forget(arc);
}

// impl for `RcWake`
unsafe fn drop_rc_raw<T: RcWake>(data: *const ()) {
    drop(Rc::<T>::from_raw(data as *const T))
}

// FIXME: panics on Rc::clone / refcount changes could wreak havoc on the
// code here. We should guard against this by aborting.
// Utility
unsafe fn increase_refcount<T: RcWake>(data: *const ()) {
    // Retain Rc by creating a copy
    let rc: Rc<T> = Rc::from_raw(data as *const T);
    let rc_clone = rc.clone();
    // Forget the Rcs again, so that the refcount isn't decreased
    mem::forget(rc);
    mem::forget(rc_clone);
}
