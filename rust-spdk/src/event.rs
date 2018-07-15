use failure::Error;
use generated::spdk_event_bindings::{
    spdk_app_fini, spdk_app_opts, spdk_app_opts_init, spdk_app_start,
};
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::ptr;

#[derive(Debug, Fail)]
enum AppError {
    #[fail(display = "Spdk failed to start: {}", _0)]
    StartupError(i32),
}

pub struct AppOpts(spdk_app_opts);

impl AppOpts {
    pub fn new() -> Self {
        let mut opts: spdk_app_opts = Default::default();
        unsafe {
            spdk_app_opts_init(&mut opts as *mut spdk_app_opts);
        }
        AppOpts(opts)
    }

    pub fn name(&mut self, name: &'static str) {
        self.0.name = CString::new(name)
            .expect("Couldn't create a string")
            .into_raw()
    }

    pub fn config_file(&mut self, config_file: &'static str) {
        self.0.config_file = CString::new(config_file)
            .expect("Couldn't create a string")
            .into_raw()
    }

    pub fn start<F>(mut self, f: F) -> Result<(), Error>
    where
        F: Fn() -> (),
    {
        let user_data = &f as *const _ as *mut c_void;

        extern "C" fn start_wrapper<F>(closure: *mut c_void, arg2: *mut c_void)
        where
            F: Fn() -> (),
        {
            let opt_closure = closure as *mut F;
            unsafe { (*opt_closure)() }
        }

        let ret = unsafe {
            let self_ref = &mut self;
            let opts_ref = &mut self_ref.0;
            spdk_app_start(
                opts_ref as *mut spdk_app_opts,
                Some(start_wrapper::<F>),
                // For now nothing to pass around
                user_data,
                ptr::null_mut(),
            )
        };

        unsafe {
            spdk_app_fini();
        }

        if ret == 0 {
            Ok(())
        } else {
            Err(AppError::StartupError(ret))?
        }
    }
}

impl Drop for AppOpts {
    fn drop(&mut self) {
        drop_if_not_null(self.0.name as *mut c_char);
        drop_if_not_null(self.0.config_file as *mut c_char);
    }
}

fn drop_if_not_null(string: *mut c_char) {
    if !string.is_null() {
        unsafe { CString::from_raw(string as *mut c_char) };
    }
}
