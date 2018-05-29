#[allow(non_upper_case_globals, non_camel_case_types)]
pub mod spdk {
    include!(concat!(env!("OUT_DIR"), "/spdk_bindings.rs"));
}
