use std::fmt::Debug;
use std::path::PathBuf;

pub use descriptor::Descriptor;

pub mod commitlog;
pub mod flush_queue;
pub mod segment;
pub mod segment_manager;

mod descriptor;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Commitlog has been shut down. Cannot add data")]
    Closed,

    #[error(
    display = "Mutation of {:?} bytes is too large for the maxiumum size of {:?}",
    size,
    max_size
    )]
    MutationTooLarge { size: u64, max_size: u64 },

    #[error(display = "IO Error: _1")]
    IO(#[error(cause)] std::io::Error),

    #[error(display = "Something else failed: _1")]
    Other(Box<dyn std::error::Error>),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(f: std::io::Error) -> Self {
        Error::IO(f)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(f: Box<dyn std::error::Error>) -> Self {
        Error::Other(f)
    }
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

    max_reserve_segments: usize,

    // Max active writes/flushes. Default value
    // zero means try to figure it out ourselves
    // uint64_t max_active_writes = 0;
    max_active_flushes: usize,
}

impl ConfigBuilder {
    fn default_max_reserve_segments() -> usize {
        12
    }
    fn default_max_active_flushes() -> usize {
        // 5 * smp::count
        0
    }
}

pub type SegmentId = u64;
pub type Position = u64;

#[derive(Eq, PartialEq, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct ReplayPosition {
    id: SegmentId,
    position: Position,
}

impl ReplayPosition {
    pub fn create(id: SegmentId, position: Position) -> Self {
        ReplayPosition { id, position }
    }
}
