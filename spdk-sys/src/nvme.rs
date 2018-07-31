use crate::generated::spdk_nvme_bindings;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

pub struct EnvOpts(spdk_nvme_bindings::spdk_env_opts);

impl EnvOpts {
    pub fn new() -> Self {
        let mut opts: spdk_nvme_bindings::spdk_env_opts = Default::default();
        unsafe {
            spdk_nvme_bindings::spdk_env_opts_init(
                &mut opts as *mut spdk_nvme_bindings::spdk_env_opts,
            );
        }
        EnvOpts(opts)
    }

    pub fn name(&mut self, name: &'static str) {
        self.0.name = CString::new(name)
            .expect("Couldn't create a string")
            .into_raw()
    }

    pub fn shm_id(&mut self, shm_id: i32) {
        self.0.shm_id = shm_id;
    }

    pub fn init(&self) -> Result<Env, ()> {
        if init_env(self) < 0 {
            return Err(());
        }
        Ok(Env {})
    }
}

impl Drop for EnvOpts {
    fn drop(&mut self) {
        if unsafe { CStr::from_ptr(self.0.name).to_str().unwrap() != "spdk" } {
            unsafe { CString::from_raw(self.0.name as *mut c_char) };
        }
    }
}

pub struct Env {}

impl Env {}

fn init_env(opts: &EnvOpts) -> c_int {
    unsafe {
        spdk_nvme_bindings::spdk_env_init(&opts.0 as *const spdk_nvme_bindings::spdk_env_opts)
    }
}
