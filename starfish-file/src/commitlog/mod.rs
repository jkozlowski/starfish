use std::path::PathBuf;
use std::fmt;

pub mod segment;
pub mod segment_manager;

static SEPARATOR: &str = "-";
static FILENAME_PREFIX: &str = "CommitLog";
static FILENAME_EXTENSION: &str = ".log";

pub enum Version {
    V1
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Version::V1 => write!(f, "1"),
        }   
    }
}

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
    
    #[builder(default = "self.default_fname_prefix()")]
    fname_prefix: String
}

impl ConfigBuilder {
    fn default_fname_prefix(&self) -> String {
        format!("{}{}", FILENAME_PREFIX, SEPARATOR)
    }
}

pub type SegmentId = u64;
pub type Position = u32;

pub struct Descriptor {
    segment_id: SegmentId,
    filename: String
}

impl Descriptor {
    pub fn create(segment_id: SegmentId, filename_prefix: &str) -> Self {
        let filename = format!("{}{}{}", filename_prefix, Version::V1, FILENAME_EXTENSION);
        Descriptor {
            segment_id,
            filename
        }
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
}
