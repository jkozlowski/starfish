use libc::c_int;
use libc::c_void;

use crate::generated::{spdk_poller, spdk_poller_register, spdk_poller_unregister};

pub struct PollerHandle {
    pub(crate) poller: *mut spdk_poller,
    #[allow(dead_code)]
    pub(crate) closure: Box<dyn Fn() -> bool>,
}

impl Drop for PollerHandle {
    #[allow(clippy::cast_ptr_alignment)]
    fn drop(&mut self) {
        let tmp_poller = self.poller;
        // This is rather dogdy, spdk_poller_unregister will write NULL to self.poller,
        // hopefully that isn't going to crash!
        unsafe { spdk_poller_unregister(tmp_poller as *mut *mut spdk_poller) }
    }
}

/// Registers a poller with spdk.
/// f: should return true if any work was done
pub fn poller_register<F>(f: F) -> PollerHandle
where
    F: Fn() -> bool + 'static,
{
    extern "C" fn poller_wrapper<F>(closure: *mut c_void) -> c_int
    where
        F: Fn() -> bool,
    {
        let opt_closure = closure as *mut F;
        let work_done = unsafe { (*opt_closure)() };
        if work_done {
            1
        } else {
            0
        }
    }

    let f_raw = Box::into_raw(Box::new(f)) as *mut dyn Fn() -> bool;
    let f_pointer = f_raw as *const _ as *mut c_void;
    let poller = unsafe { spdk_poller_register(Some(poller_wrapper::<F>), f_pointer, 0) };
    PollerHandle {
        // TODO: handle failure
        poller,
        closure: unsafe { Box::from_raw(f_raw) },
    }
}
