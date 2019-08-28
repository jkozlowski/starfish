use crate::commitlog::segment_manager::Error;
use crate::commitlog::segment_manager::SegmentManager;
use crate::fs::File;
use crate::shared::Shared;
use std::rc::Rc;

// A single commit log file on disk.
#[derive(Clone)]
pub struct Segment {
    inner: Shared<Inner>,
}

struct Inner {
    segment_manager: SegmentManager,

    file: File,

    closed: bool,

    file_pos: u64,
    flush_pos: u64,
    buf_pos: u64,
}

impl Segment {
    pub fn create(segment_manager: SegmentManager, file: File) -> Self {
        Segment {
            inner: Shared::new(Inner {
                segment_manager,

                file,

                closed: false,

                file_pos: 0,
                flush_pos: 0,
                buf_pos: 0,
            }),
        }
    }

    pub fn reset_sync_time(&self) {
        let inner = self.inner.borrow_mut();
        inner.reset_sync_time()
    }

    pub fn is_still_allocating(&self) -> bool {
        let inner = self.inner.borrow();
        inner.is_still_allocating()
    }
}

impl Inner {
    pub fn is_still_allocating(&self) -> bool {
        !self.closed && self.position() < self.segment_manager.max_size()
    }

    fn position(&self) -> u64 {
        self.file_pos + self.buf_pos
    }

    pub fn reset_sync_time(&self) {
        // DO
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        // TODO(jakubk): Make sure stuff gets closed and deleted
    }
}
