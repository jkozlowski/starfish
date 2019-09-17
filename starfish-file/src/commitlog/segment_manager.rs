use std::cmp;
use std::fs::OpenOptions;

use futures::future::poll_fn;
use futures_intrusive::sync::Semaphore;
use slog::Logger;
use tokio_sync::Lock;
use tokio_sync::mpsc;

use crate::commitlog::Config;
use crate::commitlog::Descriptor;
use crate::commitlog::Error;
use crate::commitlog::Position;
use crate::commitlog::Result;
use crate::commitlog::segment::Segment;
use crate::commitlog::SegmentId;
use crate::fs::FileSystem;
use crate::Shared;
use crate::spawn;

struct Stats {
    flush_count: u64,

    segments_created: u64,
    bytes_slack: u64,

    pending_flushes: u64,
    flush_limit_exceeded: u64,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            flush_count: 0,

            segments_created: 0,
            bytes_slack: 0,

            pending_flushes: 0,
            flush_limit_exceeded: 0,
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

pub struct FlushGuard {
    segment_manager: SegmentManager
}

impl Drop for FlushGuard {
    fn drop(&mut self) {
        //_flush_semaphore.signal();
        //--totals.pending_flushes;
    }
}

struct Inner {
    cfg: Config,

    fs: FileSystem,
    log: Logger,

    flush_semaphore: Semaphore,

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
        let max_active_flushes = cfg.max_active_flushes;

        let segment_manager = SegmentManager {
            inner: Shared::new(Inner {
                cfg,

                fs,

                log,

                flush_semaphore: Semaphore::new(false, max_active_flushes),

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
        let segment = self.active_segment().await?;
        Ok(())
    }

    pub async fn active_segment(&self) -> Result<Segment> {
        if let Some(active_segment) = self.current_segment() {
            return Ok(active_segment.clone());
        }

        let mut lock = self.inner.new_segments.clone();
        let mut locked = lock.lock().await;

        if let Some(active_segment) = self.current_segment() {
            return Ok(active_segment.clone());
        }

        self.new_segment(&mut *locked).await
    }

    pub fn max_size(&self) -> u64 {
        self.inner.max_size
    }

    pub fn sanity_check_size(&self, size: u64) -> Result<()> {
        let max_size = self.inner.max_mutation_size;
        if size > max_size {
            return Err(Error::MutationTooLarge { size, max_size });
        }
        Ok(())
    }

    pub fn record_slack(&self, slack: usize) {
        self.inner.borrow_mut().stats.bytes_slack += slack as u64;
        self.account_memory_usage(slack);
    }

    pub fn record_flush_success(&self) {
        self.inner.borrow_mut().stats.flush_count += 1;
    }

    pub fn account_memory_usage(&self, size: usize) {
        // request_controller.consume(size);
    }

    // TODO(jkozlowski): This should be some sort of lock-like API
    pub async fn begin_flush(&self) -> FlushGuard {
        self.inner.borrow_mut().stats.pending_flushes += 1;
        if self.inner.stats.pending_flushes >= self.inner.cfg.max_active_flushes as u64 {
            self.inner.borrow_mut().stats.flush_limit_exceeded += 1;
            trace!(self.inner.log,
                   "Flush ops overflow. Will block.";
                   "pending_flushes" => self.inner.stats.pending_flushes);
        }
//        if (totals.pending_flushes >= cfg.max_active_flushes) {
//            + + totals.flush_limit_exceeded;
//            clogger.trace("Flush ops overflow: {}. Will block.", totals.pending_flushes);
//        }
//        return _flush_semaphore.wait();
        unimplemented!();
    }

    async fn allocate_segment(&self) -> Result<Segment> {
        let new_segment_id = self.next_segment_id();

        let descriptor = Descriptor::create(new_segment_id);

        let mut path = self.inner.cfg.commit_log_location.clone();
        path.push(descriptor.filename());

        let mut open_options = OpenOptions::new();
        open_options.write(true).create_new(true);

        let mut file = self.inner.fs.open(path, open_options).await?;

        file.truncate(self.inner.max_size).await?;

        let segment = Segment::create(self.clone(), self.inner.log.clone(), descriptor, file);

        self.inner.borrow_mut().stats.segments_created += 1;

        Ok(segment)
    }

    async fn new_segment(&self, new_segments: &mut mpsc::Receiver<Segment>) -> Result<Segment> {
        if self.inner.shutdown {
            return Err(Error::Closed);
        }

        self.inner.borrow_mut().new_counter += 1;

        let new_segment = new_segments.recv().await.ok_or(Error::Closed)?;
        new_segment.reset_sync_time();

        self.inner.borrow_mut().segments.push(new_segment.clone());
        Ok(new_segment)
    }

    async fn replenish_reserve(manager: SegmentManager, mut tx: mpsc::Sender<Segment>) {
        async fn send_one(
            manager: &SegmentManager,
            tx: &mut mpsc::Sender<Segment>,
        ) -> std::result::Result<(), ()> {
            poll_fn(|cx| tx.poll_ready(cx)).await.map_err(|_| ())?;
            let segment = manager.allocate_segment().await.map_err(|_| ())?;
            info!(manager.inner.borrow().log,
                  "Created segment";
                  &segment);
            tx.try_send(segment).map_err(|_| ())
        }

        while let Ok(()) = send_one(&manager, &mut tx).await {
            // Successful
        }
    }

    fn current_segment(&self) -> Option<Segment> {
        self.inner.segments.last().cloned()
    }

    fn next_segment_id(&self) -> SegmentId {
        let next_segment_id = self.inner.next_segment_id;
        self.inner.borrow_mut().next_segment_id += 1;
        next_segment_id
    }
}
