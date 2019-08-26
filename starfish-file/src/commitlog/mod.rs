use simple_error::bail;
use simple_error::try_with;
use std::convert::TryFrom;
use std::path::PathBuf;
use std::str::FromStr;

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

// impl TryFrom<&str> for SegmentId {
//     type Error = Box<dyn std::error::Error>;

//     fn try_from(s: &str) -> Result<Self, Self::Error> {
//         let value = try_with!(u64::from_str(s), "Failed to parse version");

//         if value != 1 {
//             bail!("Only V1 supported: {}", value)
//         } else {
//             Ok(Version::V1)
//         }
//     }
// }

mod test {
    #[tokio::test]
    async fn my_test() {
        // let addr = "127.0.0.1:8080".parse().unwrap();
        // let mut listener = TcpListener::bind(&addr).unwrap();
        // let addr = listener.local_addr().unwrap();

        // // Connect to the listener
        // TcpStream::connect(&addr).await.unwrap();
    }
}
