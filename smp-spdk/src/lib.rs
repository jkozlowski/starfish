#[allow(non_upper_case_globals, non_camel_case_types, unused)]
mod spdk_bindings {
    include!(concat!(env!("OUT_DIR"), "/spdk_bindings.rs"));
}

pub struct EnvOpts(spdk_bindings::spdk_env_opts);

impl EnvOpts {
    pub fn new() -> Self {
        let mut opts: spdk_bindings::spdk_env_opts = Default::default();
        unsafe {
            spdk_bindings::spdk_env_opts_init(&mut opts as *mut spdk_bindings::spdk_env_opts);
        }
        EnvOpts(opts)
    }
}

pub fn init_env(opts: &EnvOpts) {
    unsafe {
        spdk_bindings::spdk_env_init(&opts.0 as *const spdk_bindings::spdk_env_opts);
    }
}
