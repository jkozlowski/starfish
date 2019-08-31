use crate::commitlog::segment_manager::SegmentManager;
use crate::commitlog::Result;
use crate::fs::File;
use crate::shared::Shared;
use crate::spawn;
use bytes::BufMut;
use bytes::BytesMut;
use std::boxed::Box;
use std::mem::size_of;
use std::pin::Pin;

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

    buffer: BytesMut,

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

                buffer: BytesMut::new(),

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

    async fn allocate<W>(&mut self, this: Segment, size: u64, writer: &W) -> Result<()>
    where
        W: Fn(BytesMut),
    {
        let total_size = size + ENTRY_OVERHEAD_SIZE as u64;
        self.segment_manager.sanity_check_size(total_size)?;

        if !self.is_still_allocating()
            || self.position() + total_size > self.segment_manager.max_size()
        {
            // would we make the file too big?
            let segment = self.finish_and_get_new(this).await?;
            // https://github.com/rust-lang/rust/issues/53690
            // https://github.com/rust-lang/rust/issues/62284
            // https://www.reddit.com/r/rust/comments/cbdxxm/why_are_recursive_async_fns_forbidden/
            let recurse: Pin<Box<dyn std::future::Future<Output = Result<()>>>> =
                Box::pin(async move {
                    let mut inner = segment.inner.borrow_mut();
                    inner.allocate(segment.clone(), size, writer).await
                });
            return recurse.await;
        } else if total_size as usize > self.buffer.remaining_mut() {
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
            return Ok(());
        }
        Ok(())
    }

    pub fn reset_sync_time(&self) {
        // DO
    }

    fn position(&self) -> u64 {
        self.file_pos + self.buf_pos
    }

    async fn finish_and_get_new(&mut self, this: Segment) -> Result<Segment> {
        self.closed = true;
        spawn(async move {
            let inner = this.inner.borrow();
            inner.sync(false).await.map_err(|e| ());
        });
        self.segment_manager.active_segment().await
    }

    async fn sync(&self, shutdown: bool) -> Result<Segment> {
        /*
         * If we are shutting down, we first
         * close the allocation gate, thus no new
         * data can be appended. Then we just issue a
         * flush, which will wait for any queued ops
         * to complete as well. Then we close the ops
         * queue, just to be sure.
         */
        // if (shutdown) {
        //     auto me = shared_from_this();
        //     return _gate.close().then([me] {
        //         me->_closed = true;
        //         return me->sync().finally([me] {
        //             // When we get here, nothing should add ops,
        //             // and we should have waited out all pending.
        //             return me->_pending_ops.close().finally([me] {
        //                 return me->_file.truncate(me->_flush_pos).then([me] {
        //                     return me->_file.close();
        //                 });
        //             });
        //         });
        //     });
        // }

        // // Note: this is not a marker for when sync was finished.
        // // It is when it was initiated
        // reset_sync_time();
        // return cycle(true);
        unimplemented!()
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        // TODO(jakubk): Make sure stuff gets closed and deleted
    }
}
