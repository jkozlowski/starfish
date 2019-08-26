use std::path::PathBuf;

pub mod segment;
pub mod segment_manager;

mod descriptor;
pub use descriptor::Descriptor;

#[derive(Debug, Clone, PartialEq)]
pub enum SyncMode {
    Periodic,
    Batch,
}

#[derive(Builder, Debug, PartialEq)]
pub struct Config {
    commit_log_location: PathBuf,

    #[builder(default = "100")]
    commitlog_total_space_in_mb: u64,

    #[builder(default = "32")]
    commitlog_segment_size_in_mb: u64,
    #[builder(default = "10 * 1000")]
    commitlog_sync_period_in_ms: u64,

    // // Max number of segments to keep in pre-alloc reserve.
    // // Not (yet) configurable from scylla.conf.
    // uint64_t max_reserve_segments = 12;
    // // Max active writes/flushes. Default value
    // // zero means try to figure it out ourselves
    // uint64_t max_active_writes = 0;
    // uint64_t max_active_flushes = 0;
    #[builder(default = "SyncMode::Batch")]
    sync_mode: SyncMode,
}

pub type SegmentId = u64;
pub type Position = u32;
