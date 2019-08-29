use crate::commitlog::segment::Segment;
use crate::commitlog::Config;
use crate::commitlog::Descriptor;
use crate::commitlog::Error;
use crate::commitlog::Position;
use crate::commitlog::Result;
use crate::commitlog::SegmentId;
use crate::fs::FileSystem;
use crate::spawn;
use crate::Shared;
use futures::future::poll_fn;
use futures::TryStreamExt;

use slog::Logger;
use std::cmp;


use std::fs::OpenOptions;

use tokio_sync::mpsc;
use tokio_sync::Lock;


struct Stats {
    segments_created: u64,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            segments_created: 0,
        }
    }
}

pub trait EntryWriter: Sized {
    // How much to write
    fn size() -> usize;
    // virtual size_t size(segment&) = 0;
    //     // Returns segment-independent size of the entry. Must be <= than segment-dependant size.
    //     virtual size_t size() = 0;
    //     virtual void write(segment&, output&) = 0;
}

#[derive(Clone)]
pub struct SegmentManager {
    inner: Shared<Inner>,
}

struct Inner {
    cfg: Config,

    fs: FileSystem,
    log: Logger,

    segments: Vec<Segment>,

    new_segments: Lock<mpsc::Receiver<Segment>>,

    max_size: u64,
    max_mutation_size: u64,

    new_counter: u64,
    next_segment_id: SegmentId,

    stats: Stats,

    shutdown: bool,
}

impl SegmentManager {
    pub async fn create(cfg: Config, fs: FileSystem, log: Logger) -> Result<SegmentManager> {
        let max_size = cmp::min(
            u64::from(Position::max_value()),
            cmp::max(cfg.commitlog_segment_size_in_mb, 1) * 1024 * 1024,
        );

        let (tx, rx) = mpsc::channel(cfg.max_reserve_segments);

        let segment_manager = SegmentManager {
            inner: Shared::new(Inner {
                cfg,

                fs,

                log,

                segments: vec![],
                new_segments: Lock::new(rx),

                max_size,
                max_mutation_size: max_size >> 1,

                new_counter: 0,
                next_segment_id: 0,

                stats: Default::default(),

                shutdown: false,
            }),
        };

        spawn(SegmentManager::replenish_reserve(
            segment_manager.clone(),
            tx,
        ));

        Ok(segment_manager)
    }

    pub async fn allocate_when_possible(&self) -> Result<()> {
        let mut inner = self.inner.borrow_mut();
        let _segment = inner.active_segment().await?;
        Ok(())
    }

    pub fn max_size(&self) -> u64 {
        self.inner.max_size()
    }

    async fn allocate_segment(&self) -> Result<Segment> {
        let mut inner = self.inner.borrow_mut();
        inner.allocate_segment(self).await
    }

    async fn replenish_reserve(manager: SegmentManager, mut tx: mpsc::Sender<Segment>) {
        async fn send_one(
            manager: &SegmentManager,
            tx: &mut mpsc::Sender<Segment>,
        ) -> std::result::Result<(), ()> {
            poll_fn(|cx| tx.poll_ready(cx)).await.map_err(|_| ())?;
            let segment = manager.allocate_segment().await.map_err(|_| ())?;
            info!(manager.inner.borrow().log, "Created segment");
            tx.try_send(segment).map_err(|_| ())
        }

        while let Ok(()) = send_one(&manager, &mut tx).await {
            // Successful
        }
    }
}

impl Inner {
    fn max_size(&self) -> u64 {
        self.max_size
    }

    async fn active_segment(&mut self) -> Result<Segment> {
        if let Some(active_segment) = self.current_segment() {
            return Ok(active_segment.clone());
        }

        let mut locked = self.new_segments.lock().await;

        if let Some(active_segment) = self.current_segment() {
            return Ok(active_segment.clone());
        }

        self.new_segment(&mut *locked).await
    }

    fn current_segment(&self) -> Option<Segment> {
        self.segments.last().cloned()
    }

    async fn allocate_segment(&mut self, this: &SegmentManager) -> Result<Segment> {
        let new_segment_id = self.next_segment_id();

        let descriptor = Descriptor::create(new_segment_id);

        let mut path = self.cfg.commit_log_location.clone();
        path.push(descriptor.filename());

        let mut open_options = OpenOptions::new();
        open_options.write(true).create_new(true);

        let mut file = self.fs.open(path, open_options).await?;

        file.truncate(self.max_size).await?;

        let segment = Segment::create(this.clone(), file);

        self.stats.segments_created += 1;

        Ok(segment)
    }

    async fn new_segment(&mut self, new_segments: &mut mpsc::Receiver<Segment>) -> Result<Segment> {
        if self.shutdown {
            return Err(Error::Closed);
        }

        self.new_counter += 1;

        let new_segment = new_segments.recv().await.ok_or(Error::Closed)?;
        new_segment.reset_sync_time();

        self.segments.push(new_segment.clone());
        Ok(new_segment)
    }

    fn next_segment_id(&mut self) -> SegmentId {
        let next_segment_id = self.next_segment_id;
        self.next_segment_id += 1;
        next_segment_id
    }
}
