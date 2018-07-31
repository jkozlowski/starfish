#[allow(
    rust_2018_idioms,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unused,
    elided_lifetimes_in_paths
)]
pub mod spdk_event_bindings {
    include!(concat!(env!("OUT_DIR"), "/spdk_event_bindings.rs"));
}

#[allow(
    rust_2018_idioms,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unused,
    elided_lifetimes_in_paths
)]
pub mod spdk_nvme_bindings {
    include!(concat!(env!("OUT_DIR"), "/spdk_nvme_bindings.rs"));
}

#[allow(
    rust_2018_idioms,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unused,
    elided_lifetimes_in_paths
)]
pub mod spdk_io_channel_bindings {
    include!(concat!(env!("OUT_DIR"), "/spdk_io_channel_bindings.rs"));
}

#[allow(
    rust_2018_idioms,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unused,
    elided_lifetimes_in_paths
)]
pub mod spdk_bdev_bindings {
    include!(concat!(env!("OUT_DIR"), "/spdk_bdev_bindings.rs"));
}

#[allow(
    rust_2018_idioms,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unused,
    elided_lifetimes_in_paths
)]
pub mod spdk_blob_bdev_bindings {
    include!(concat!(env!("OUT_DIR"), "/spdk_blob_bdev_bindings.rs"));
}

#[allow(
    rust_2018_idioms,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unused,
    elided_lifetimes_in_paths
)]
pub mod spdk_blob_bindings {
    include!(concat!(env!("OUT_DIR"), "/spdk_blob_bindings.rs"));
}
