use crate::commitlog::segment::Segment;
use crate::commitlog::Config;
use crate::commitlog::Descriptor;
use crate::commitlog::Position;
use crate::commitlog::SegmentId;
use crate::fs::FileSystem;
use crate::Shared;
use std::cmp;
use std::fs::OpenOptions;
use std::rc::Rc;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Commitlog has been shut down. Cannot add data")]
    Closed,

    #[error(display = "IO Error: _1")]
    IO(crate::fs::Error),
}

impl From<crate::fs::Error> for Error {
    fn from(f: crate::fs::Error) -> Self {
        Error::IO(f)
    }
}

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

#[derive(Clone)]
pub struct SegmentManager {
    inner: Shared<Inner>,
}

struct Inner {
    cfg: Config,

    fs: FileSystem,

    segments: Vec<Rc<Segment>>,

    max_size: u64,
    max_mutation_size: u64,

    new_counter: u64,
    next_segment_id: SegmentId,

    stats: Stats,

    shutdown: bool,
}

impl SegmentManager {
    pub async fn create(cfg: Config, fs: FileSystem) -> Result<SegmentManager, Error> {
        let max_size = cmp::min(
            u64::from(Position::max_value()),
            cmp::max(cfg.commitlog_segment_size_in_mb, 1) * 1024 * 1024,
        );
        Ok(SegmentManager {
            inner: Shared::new(Inner {
                cfg,

                fs,

                segments: vec![],

                max_size,
                max_mutation_size: max_size >> 1,

                new_counter: 0,
                next_segment_id: 0,

                stats: Default::default(),

                shutdown: false,
            }),
        })
    }

    pub async fn allocate_when_possible(&self) -> Result<(), ()> {
        let mut inner = self.inner.borrow_mut();
        let segment = inner.active_segment().await?;
        Ok(())
    }

    async fn allocate_segment(&self) -> Result<Segment, Error> {
        let mut inner = self.inner.borrow_mut();
        inner.allocate_segment(self.clone(), true).await
    }

    pub fn max_size(&self) -> u64 {
        self.inner.max_size()
    }
}

impl Inner {
    fn max_size(&self) -> u64 {
        self.max_size
    }

    async fn active_segment(&mut self) -> Result<Rc<Segment>, ()> {
        let active_segment = self
            .segments
            .last()
            .filter(|segment| segment.is_still_allocating())
            .unwrap()
            .clone();
        Ok(active_segment)
    }

    async fn new_segment(&mut self) -> Result<Rc<Segment>, Error> {
        if self.shutdown {
            return Err(Error::Closed);
        }

        self.new_counter += 1;

        unimplemented!()
    }

    async fn allocate_segment(
        &mut self,
        this: SegmentManager,
        active: bool,
    ) -> Result<Segment, Error> {
        let new_segment_id = self.next_segment_id();

        let descriptor = Descriptor::create(new_segment_id, &self.cfg.fname_prefix);

        let mut path = self.cfg.commit_log_location.clone();
        path.push(descriptor.filename());

        let mut open_options = OpenOptions::new();
        open_options.write(true).create_new(true);

        let file = self.fs.open(path, open_options).await?;

        file.truncate(self.max_size).await?;

        let segment = Segment::create(this, file);

        self.stats.segments_created += 1;

        Ok(segment)
    }

    fn next_segment_id(&mut self) -> SegmentId {
        let next_segment_id = self.next_segment_id;
        self.next_segment_id += 1;
        return next_segment_id;
    }
}
