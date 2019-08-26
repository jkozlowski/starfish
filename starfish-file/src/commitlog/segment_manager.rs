use crate::commitlog::segment::Segment;
use crate::commitlog::Config;
use crate::commitlog::Descriptor;
use crate::commitlog::Position;
use crate::commitlog::SegmentId;
use crate::fs::FileSystem;
use crate::Shared;
use futures::TryStreamExt;
use std::cmp;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::fs::OpenOptions;
use std::rc::Rc;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Commitlog has been shut down. Cannot add data")]
    Closed,

    #[error(display = "IO Error: _1")]
    IO(std::io::Error),

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

    segments: Vec<Segment>,

    max_size: u64,
    max_mutation_size: u64,

    new_counter: u64,
    next_segment_id: SegmentId,

    stats: Stats,

    shutdown: bool,
}

impl SegmentManager {
    pub async fn create(cfg: Config, fs: FileSystem) -> Result<SegmentManager> {
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

    pub async fn init(&mut self) -> Result<()> {
        unimplemented!()
    }

    pub async fn allocate_when_possible(&self) -> Result<()> {
        let mut inner = self.inner.borrow_mut();
        let segment = inner.active_segment().await?;
        Ok(())
    }

    pub fn max_size(&self) -> u64 {
        self.inner.max_size()
    }

    async fn allocate_segment(&self) -> Result<Segment> {
        let mut inner = self.inner.borrow_mut();
        inner.allocate_segment(self.clone(), true).await
    }

    async fn list_descriptors(&self) -> Result<Descriptor> {
        let mut inner = self.inner.borrow_mut();

        let commit_log_location = &inner.cfg.commit_log_location;

        let mut files = inner.fs.read_dir(commit_log_location.clone()).await?;

        let mut descriptors: Vec<Descriptor> = vec![];

        for elem in files.try_next().await? {
            let os_file_name = elem.file_name();
            let file_name = os_file_name.to_str().expect("Failed to get file name");
            let descriptor = Descriptor::try_create(file_name)?;
        }

        unimplemented!();
    }
    // list_descriptors(sstring dirname)
}

impl Inner {
    fn max_size(&self) -> u64 {
        self.max_size
    }

    async fn active_segment(&mut self) -> Result<Segment> {
        let active_segment = self
            .segments
            .last()
            .filter(|segment| segment.is_still_allocating())
            .unwrap()
            .clone();
        Ok(active_segment)
    }

    async fn new_segment(&mut self) -> Result<Segment> {
        if self.shutdown {
            return Err(Error::Closed);
        }

        self.new_counter += 1;

        unimplemented!()
    }

    async fn allocate_segment(&mut self, this: SegmentManager, active: bool) -> Result<Segment> {
        let new_segment_id = self.next_segment_id();

        let descriptor = Descriptor::create(new_segment_id);

        let mut path = self.cfg.commit_log_location.clone();
        path.push(descriptor.filename());

        let mut open_options = OpenOptions::new();
        open_options.write(true).create_new(true);

        let mut file = self.fs.open(path, open_options).await?;

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
