use crate::waker::clone_rc_raw;
use crate::waker::wake_by_ref_rc_raw;
use crate::waker::RcWake;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use std::task::RawWaker;
use std::task::RawWakerVTable;
use std::task::Waker;

/// A [`Waker`] that is only valid for a given lifetime.
///
/// Note: this type implements [`Deref<Target = Waker>`](std::ops::Deref),
/// so it can be used to get a `&Waker`.
#[derive(Debug)]
pub struct WakerRef<'a> {
    waker: Waker,
    _marker: PhantomData<&'a ()>,
}

impl WakerRef<'_> {
    /// Create a new [`WakerRef`] from a [`Waker`].
    ///
    /// Note: this function is safe, but it is generally only used
    /// from `unsafe` contexts that need to create a `Waker`
    /// that is guaranteed not to outlive a particular lifetime.
    pub fn new(waker: Waker) -> Self {
        WakerRef {
            waker,
            _marker: PhantomData,
        }
    }
}

impl Deref for WakerRef<'_> {
    type Target = Waker;

    fn deref(&self) -> &Waker {
        &self.waker
    }
}

#[inline]
unsafe fn noop(_data: *const ()) {}

unsafe fn wake_unreachable(_data: *const ()) {
    // With only a reference, calling `wake_rc_raw()` would be unsound,
    // since the `WakerRef` didn't increment the refcount of the `RcWake`,
    // and `wake_rc_raw` would *decrement* it.
    //
    // This should never be reachable, since `WakerRef` only provides a `Deref`
    // to the inner `Waker`.
    //
    // Still, safer to panic here than to call `wake_rc_raw`.
    unreachable!("WakerRef::wake");
}

/// Creates a reference to a [`Waker`] from a reference to `Rc<impl RcWake>`.
///
/// The resulting [`Waker`] will call
/// [`RcWake.wake()`](RcWake::wake) if awoken.
#[inline]
pub fn waker_ref<W>(wake: &Rc<W>) -> WakerRef<'_>
where
    W: RcWake,
{
    // This uses the same mechanism as Arc::into_raw, without needing a reference.
    // This is potentially not stable
    #![allow(clippy::cast_ptr_alignment)]
    let ptr = &*wake as &W as *const W as *const ();

    // Similar to `waker_vtable`, but with a no-op `drop` function.
    // Clones of the resulting `RawWaker` will still be dropped normally.
    let vtable = &RawWakerVTable::new(
        clone_rc_raw::<W>,
        wake_unreachable,
        wake_by_ref_rc_raw::<W>,
        noop,
    );

    let waker = unsafe { Waker::from_raw(RawWaker::new(ptr, vtable)) };
    WakerRef::new(waker)
}
