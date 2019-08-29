use crate::commitlog::segment_manager::SegmentManager;
use crate::commitlog::Config;
use crate::commitlog::Result;
use crate::fs::FileSystem;
use slog::Logger;

pub struct Commitlog {
    segment_manager: SegmentManager,
}

impl Commitlog {
    pub async fn create(_cfg: Config, _fs: FileSystem, _log: Logger) -> Result<Commitlog> {
        unimplemented!()
    }
}
