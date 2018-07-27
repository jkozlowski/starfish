use std::os::raw::{c_int, c_void};

use crate::generated::spdk_io_channel_bindings::{ spdk_poller_register };

/// Registers a poller with spdk.
/// f: should return true if any work was done
pub fn poller_register<F>(f: F) where F: Fn() -> bool {
    extern "C" fn poller_wrapper<F>(closure: *mut c_void) -> c_int
    where
        F: Fn() -> bool,
    {
        let opt_closure = closure as *mut F;
        let work_done = unsafe { (*opt_closure)() };
        if work_done {
            return 1;
        } else {
            return 0;
        }
    }

    let f_pointer = Box::into_raw(Box::new(f)) as *const _ as *mut c_void;
    unsafe {
        spdk_poller_register(
            Some(poller_wrapper::<F>),
            f_pointer,
            0
        )
    };
}
