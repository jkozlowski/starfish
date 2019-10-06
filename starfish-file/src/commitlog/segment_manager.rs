use std::cmp;
use std::fs::OpenOptions;

use crate::commitlog::segment;
use crate::future::gate::Gate;
use crate::future::semaphore::Semaphore;
use crate::future::semaphore::SemaphoreGuard;
use futures::future::poll_fn;
use slog::Logger;
use std::time::Duration;
use tokio_sync::mpsc;
use tokio_sync::Mutex;

use crate::commitlog::segment::Segment;
use crate::commitlog::Descriptor;
use crate::commitlog::Error;
use crate::commitlog::Position;
use crate::commitlog::Result;
use crate::commitlog::SegmentId;
use crate::commitlog::{Config, ReplayPositionHolder};
use crate::fs::FileSystem;
use crate::spawn;
use crate::Shared;
use bytes::BytesMut;
use tokio::timer;

struct Stats {
    cycle_count: u64,
    flush_count: u64,
    allocation_count: u64,
    bytes_written: u64,
    bytes_slack: u64,
    segments_created: u64,
    segments_destroyed: u64,
    pending_flushes: u64,
    flush_limit_exceeded: u64,
    total_size: u64,
    buffer_list_bytes: u64,
    total_size_on_disk: u64,
    requests_blocked_memory: u64,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            cycle_count: 0,
            flush_count: 0,
            allocation_count: 0,
            bytes_written: 0,
            bytes_slack: 0,
            segments_created: 0,
            segments_destroyed: 0,
            pending_flushes: 0,
            flush_limit_exceeded: 0,
            total_size: 0,
            buffer_list_bytes: 0,
            total_size_on_disk: 0,
            requests_blocked_memory: 0,
        }
    }
}

#[derive(Clone)]
pub struct SegmentManager {
    inner: Shared<Inner>,
}

pub struct FlushGuard {
    inner: Shared<Inner>,
    guard: SemaphoreGuard,
}

impl Drop for FlushGuard {
    fn drop(&mut self) {
        self.inner.borrow_mut().stats.pending_flushes -= 1;
    }
}

struct Inner {
    cfg: Config,

    fs: FileSystem,
    log: Logger,

    flush_semaphore: Semaphore,

    request_controller: Semaphore,

    segments: Vec<Segment>,

    new_segments: Shared<Mutex<mpsc::Receiver<Segment>>>,

    max_size: u64,
    max_mutation_size: u64,
    max_disk_size: u64, // per-shard

    gate: Gate,
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
        let max_mutation_size = max_size >> 1;

        let (tx, rx) = mpsc::channel(cfg.max_reserve_segments);
        let max_active_flushes = cfg.max_active_flushes;

        // That is enough concurrency to allow for our largest mutation (max_mutation_size), plus
        // an existing in-flight buffer. Since we'll force the cycling() of any buffer that is bigger
        // than default_size at the end of the allocation, that allows for every valid mutation to
        // always be admitted for processing.
        let max_request_controller_units = max_mutation_size as usize + segment::DEFAULT_SIZE;

        // Divide the size-on-disk threshold by #cpus used, since we assume
        // we distribute stuff more or less equally across shards.
        let commitlog_total_space_in_mb = cfg.commitlog_total_space_in_mb as f64;
        let smp_count = 1 as f64; // TODO(jkozlowski): Pull out num of cores
        let max_disk_size = ((commitlog_total_space_in_mb / smp_count).ceil() as u64) * 1024 * 1024;

        let segment_manager = SegmentManager {
            inner: Shared::new(Inner {
                cfg,

                fs,

                log: log.clone(),

                flush_semaphore: Semaphore::new(max_active_flushes),

                request_controller: Semaphore::new(max_request_controller_units),

                segments: vec![],
                new_segments: Shared::new(Mutex::new(rx)),

                max_size,
                max_mutation_size,
                max_disk_size,

                gate: Gate::new(),
                new_counter: 0,
                next_segment_id: 0,

                stats: Default::default(),

                shutdown: false,
            }),
        };

        // TODO(jkozlowski): Figure out if we need a separate #init method,
        // or if doing this in constructor is fine.

        // TODO(jkozlowski): List descriptors and whatnot

        spawn(SegmentManager::replenish_reserve(
            segment_manager.clone(),
            tx,
        ));

        // TODO(jkozlowski): Finish this off
        //let delay = engine().cpu_id() * std::ceil(double(cfg.commitlog_sync_period_in_ms) / smp::count);
        let delay = Duration::from_secs(1);
        trace!(log.clone(), "Delaying timer loop"; "delay" => ?delay);

        // always run the timer now, since we need to handle segment pre-alloc etc as well.
        let segment_manager_1 = segment_manager.clone();
        spawn(async move {
            segment_manager_1.timer_loop(delay).await;
        });

        Ok(segment_manager)
    }

    pub async fn allocate_when_possible<W>(
        &self,
        size: u64,
        writer: &W,
    ) -> Result<ReplayPositionHolder>
    where
        W: Fn(BytesMut),
    {
        // If this is already too big now, we should throw early. It's also a correctness issue, since
        // if we are too big at this moment we'll never reach allocate() to actually throw at that
        // point.
        self.sanity_check_size(size)?;

        if !self.inner.request_controller.may_proceed(size as usize) {
            self.inner.borrow_mut().stats.requests_blocked_memory += 1;
        }

        let _ = self
            .inner
            .request_controller
            .wait(size as usize)
            .await
            .map_err(|broken| Error::FailedToAppend(Box::new(broken)))?;

        let segment = self.active_segment().await?;
        segment.allocate(size, writer).await
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

    pub fn record_cycle_success(&self, bytes: usize) {
        self.inner.borrow_mut().stats.bytes_written += bytes as u64;
        self.inner.borrow_mut().stats.total_size_on_disk += bytes as u64;
        self.inner.borrow_mut().stats.cycle_count += 1;
    }

    pub fn account_memory_usage(&self, size: usize) {
        // request_controller.consume(size);
    }

    pub async fn begin_flush(&self) -> Result<FlushGuard> {
        self.inner.borrow_mut().stats.pending_flushes += 1;
        if self.inner.stats.pending_flushes >= self.inner.cfg.max_active_flushes as u64 {
            self.inner.borrow_mut().stats.flush_limit_exceeded += 1;
            trace!(self.inner.log,
                   "Flush ops overflow. Will block.";
                   "pending_flushes" => self.inner.stats.pending_flushes);
        }
        let guard = self
            .inner
            .flush_semaphore
            .wait(1)
            .await
            .map_err(|broken| Error::FailedToFlush(Box::new(broken)))?;
        return Ok(FlushGuard {
            inner: self.inner.clone(),
            guard,
        });
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

        //        if (_reserve_segments.empty() && (_reserve_segments.max_size() < cfg.max_reserve_segments)) {
        //            _reserve_segments.set_max_size(_reserve_segments.max_size() + 1);
        //            clogger.debug("Increased segment reserve count to {}", _reserve_segments.max_size());
        //        }

        let new_segment = new_segments.recv().await.ok_or(Error::Closed)?;

        self.inner.borrow_mut().segments.push(new_segment.clone());
        Ok(new_segment)
    }

    async fn flush_segments(&self, force: bool) {
        if self.inner.segments.is_empty() {
            return;
        }
    }

    async fn do_pending_deletes(&self) {}

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

    /// Gets the current segment if it can still be allocated to
    fn current_segment(&self) -> Option<Segment> {
        self.inner
            .segments
            .last()
            .filter(|segment| segment.is_still_allocating())
            .cloned()
    }

    fn next_segment_id(&self) -> SegmentId {
        let next_segment_id = self.inner.next_segment_id;
        self.inner.borrow_mut().next_segment_id += 1;
        next_segment_id
    }

    async fn timer_loop(&self, initial_delay: Duration) {
        let mut initial = true;
        let period = Duration::from_millis(self.inner.cfg.commitlog_sync_period_in_ms);
        loop {
            let delay = if initial {
                initial = false;
                period + initial_delay
            } else {
                period
            };

            timer::delay_for(delay).await;

            let res = self.inner.gate.enter();
            if let Err(_) = res {
                break;
            }

            // IFF a new segment was put in use since last we checked, and we're
            // above threshold, request flush.
            if self.inner.new_counter > 0 {
                let max = self.inner.max_disk_size;
                let cur = self.inner.stats.total_size_on_disk;
                if max != 0 && cur >= max {
                    self.inner.borrow_mut().new_counter = 0;
                    debug!(self.inner.log,
                       "Size on disk exceeds local maximum";
                       "size_on_disk" => cur / (1024 * 1024),
                       "max_disk_size" => max / (1024 * 1024));
                    self.flush_segments(false).await;
                }
            }
            self.do_pending_deletes().await
        }
    }
}
