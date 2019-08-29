use crate::commitlog::segment_manager::SegmentManager;
use crate::commitlog::Result;
use crate::fs::File;
use crate::shared::Shared;
use bytes::BufMut;
use bytes::BytesMut;
use std::mem::size_of;

// The commit log entry overhead in bytes (int: length + int: head checksum + int: tail checksum)
const ENTRY_OVERHEAD_SIZE: u64 = (3 * size_of::<u32>()) as u64;
static SEGMENT_OVERHEAD_SIZE: u64 = (2 * size_of::<u32>()) as u64;
static DESCRIPTOR_HEADER_SIZE: u64 = (5 * size_of::<u32>()) as u64;
static SEGMENT_MAGIC: u32 =
    (('S' as u32) << 24) | (('C' as u32) << 16) | (('L' as u32) << 8) | ('C' as u32);

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

    pub async fn allocate<W>(&mut self, size: u64, writer: &W) -> Result<()>
    where
        W: Fn(BytesMut),
    {
        let total_size = size + ENTRY_OVERHEAD_SIZE as u64;
        self.segment_manager.sanity_check_size(total_size)?;

        if !self.is_still_allocating()
            || self.position() + total_size > self.segment_manager.max_size()
        { // would we make the file too big?
             // return finish_and_get_new(timeout).then([id, writer = std::move(writer), permit = std::move(permit), timeout] (auto new_seg) mutable {
             //     return new_seg->allocate(id, std::move(writer), std::move(permit), timeout);
             // });
        } else if (!_buffer.empty() && (s > (_buffer.size() - _buf_pos))) { // enough data?
             // if (_segment_manager->cfg.mode == sync_mode::BATCH) {
             //     // TODO: this could cause starvation if we're really unlucky.
             //     // If we run batch mode and find ourselves not fit in a non-empty
             //     // buffer, we must force a cycle and wait for it (to keep flush order)
             //     // This will most likely cause parallel writes, and consecutive flushes.
             //     return with_timeout(timeout, cycle(true)).then([this, id, writer = std::move(writer), permit = std::move(permit), timeout] (auto new_seg) mutable {
             //         return new_seg->allocate(id, std::move(writer), std::move(permit), timeout);
             //     });
             // } else {
             //     cycle().discard_result().handle_exception([] (auto ex) {
             //         clogger.error("Failed to flush commits to disk: {}", ex);
             //     });
             // }
        }
        unimplemented!()
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
