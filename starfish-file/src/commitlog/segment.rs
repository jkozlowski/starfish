use crate::commitlog::segment_manager::Error;
use crate::commitlog::segment_manager::SegmentManager;
use crate::fs::File;
use std::rc::Rc;

// A single commit log file on disk.
pub struct Segment {
    segment_manager: Rc<SegmentManager>,

    file: File,

    closed: bool,

    file_pos: u64,
    flush_pos: u64,
    buf_pos: u64,
}

impl Segment {
    pub fn create(segment_manager: Rc<SegmentManager>, file: File) -> Self {
        Segment {
            segment_manager,

            file,

            closed: false,

            file_pos: 0,
            flush_pos: 0,
            buf_pos: 0,
        }
    }

    pub fn is_still_allocating(&self) -> bool {
        !self.closed && self.position() < self.segment_manager.max_size()
    }

    fn position(&self) -> u64 {
        self.file_pos + self.buf_pos
    }
}

impl Drop for Segment {
    fn drop(&mut self) {
        // TODO(jakubk): Make sure stuff gets closed and deleted
    }
}
