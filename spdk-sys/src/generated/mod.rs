#[allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused)]
pub mod spdk_event_bindings {
    include!(concat!(env!("OUT_DIR"), "/spdk_event_bindings.rs"));
}

#[allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused)]
pub mod spdk_nvme_bindings {
    include!(concat!(env!("OUT_DIR"), "/spdk_nvme_bindings.rs"));
}
