use crate::commitlog::segment_manager::SegmentManager;
use crate::commitlog::Config;
use crate::commitlog::Result;
use crate::fs::FileSystem;
use slog::Logger;

#[derive(Clone)]
pub struct Commitlog {
    segment_manager: SegmentManager,
}

impl Commitlog {
    pub async fn create(cfg: Config, fs: FileSystem, log: Logger) -> Result<Commitlog> {
        Ok(Commitlog {
            segment_manager: SegmentManager::create(cfg, fs, log).await?,
        })
    }
}
