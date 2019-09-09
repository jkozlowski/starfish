use crate::commitlog::flush_queue::FlushQueue;
use crate::commitlog::segment_manager::SegmentManager;
use crate::commitlog::Descriptor;
use crate::commitlog::Position;
use crate::commitlog::ReplayPosition;
use crate::commitlog::Result;
use crate::fs::File;
use crate::shared::Shared;
use crate::spawn;
use bytes::BufMut;
use bytes::BytesMut;
use slog::Logger;

use std::boxed::Box;
use std::mem::size_of;
use std::pin::Pin;
use byteorder::NetworkEndian;

// The commit log entry overhead in bytes (int: length + int: head checksum + int: tail checksum)
const ENTRY_OVERHEAD_SIZE: u64 = (3 * size_of::<u32>()) as u64;
static SEGMENT_OVERHEAD_SIZE: u64 = (2 * size_of::<u32>()) as u64;
static DESCRIPTOR_HEADER_SIZE: u64 = (5 * size_of::<u32>()) as u64;
static SEGMENT_MAGIC: u32 =
    (('S' as u32) << 24) | (('C' as u32) << 16) | (('L' as u32) << 8) | ('C' as u32);
static ALIGNMENT: usize = 4096;

// A single commit log file on disk.
#[derive(Clone)]
pub struct Segment {
    inner: Shared<Inner>,
}

struct Inner {
    segment_manager: SegmentManager,
    log: Logger,

    file: File,
    descriptor: Descriptor,

    closed: bool,

    file_pos: u64,
    flush_pos: u64,

    buffer: BytesMut,
    pending_ops: FlushQueue<ReplayPosition>,

    num_allocs: u64,
}

impl Segment {
    pub fn create(
        segment_manager: SegmentManager,
        log: Logger,
        descriptor: Descriptor,
        file: File,
    ) -> Self {
        let log = log.new(o!(descriptor.clone()));
        Segment {
            inner: Shared::new(Inner {
                segment_manager,
                log,

                file,
                descriptor,

                closed: false,

                file_pos: 0,
                flush_pos: 0,

                buffer: BytesMut::new(),

                pending_ops: FlushQueue::new(),

                num_allocs: 0,
            }),
        }
    }

    pub fn reset_sync_time(&self) {
        unimplemented!();
    }

    pub fn is_still_allocating(&self) -> bool {
        !self.inner.closed && self.position() < self.inner.segment_manager.max_size()
    }

    pub async fn allocate<W>(&self, size: u64, writer: &W) -> Result<()>
        where
            W: Fn(BytesMut),
    {
        let total_size = size + ENTRY_OVERHEAD_SIZE as u64;
        self.inner.segment_manager.sanity_check_size(total_size)?;

        if !self.is_still_allocating()
            || self.position() + total_size > self.inner.segment_manager.max_size()
        {
            // would we make the file too big?
            let mut segment = self.finish_and_get_new().await?;
            // https://github.com/rust-lang/rust/issues/53690
            // https://github.com/rust-lang/rust/issues/62284
            // https://www.reddit.com/r/rust/comments/cbdxxm/why_are_recursive_async_fns_forbidden/
            let recurse: Pin<Box<dyn std::future::Future<Output=_>>> =
                Box::pin(segment.allocate(size, writer));
            return recurse.await;
        } else if total_size as usize > self.inner.buffer.remaining_mut() {
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

    /**
     * Send any buffer contents to disk and get a new tmp buffer
     */
    // See class comment for info
    async fn cycle(&self, flush_after: bool) -> Result<Segment> {
        if self.inner.buffer.is_empty() {
            return if flush_after {
                self.flush_from_start().await
            } else {
                Ok(self.clone())
            };
        }

        self.clear_buffer_slack();
        let size = self.inner.buffer.len() as u64;
        let mut buf = self.inner.borrow_mut().buffer.take();
        let off = self.inner.file_pos;
        let top = off + size;
        let num = self.inner.num_allocs;

        self.inner.borrow_mut().file_pos = top;
        self.inner.borrow_mut().num_allocs = 0;

        let header_size = 0;
        unsafe { buf.set_len(0); }

        if off == 0 {
            // first block. write file header.
            buf.put_u32_be(SEGMENT_MAGIC);
            buf.put_u32_be(self.inner.descriptor.version().into());
            buf.put_u64_be(self.inner.descriptor.segment_id());
            //     crc32_nbo crc;
            //     crc.process(_desc.ver);
            //     crc.process<int32_t>(_desc.id & 0xffffffff);
            //     crc.process<int32_t>(_desc.id >> 32);
            //     out.write(crc.checksum());
            //     header_size = descriptor_header_size;
        }

        // // write chunk header
        // crc32_nbo crc;
        // crc.process<int32_t>(_desc.id & 0xffffffff);
        // crc.process<int32_t>(_desc.id >> 32);
        // crc.process(uint32_t(off + header_size));

        // out.write(uint32_t(_file_pos));
        // out.write(crc.checksum());

        // forget_schema_versions();

        // replay_position rp(_desc.id, position_type(off));

        // clogger.trace("Writing {} entries, {} k in {} -> {}", num, size, off, off + size);

        // // The write will be allowed to start now, but flush (below) must wait for not only this,
        // // but all previous write/flush pairs.
        // return _pending_ops.run_with_ordered_post_op(rp, [this, size, off, buf = std::move(buf)]() mutable {
        //         auto written = make_lw_shared<size_t>(0);
        //         auto p = buf.get();
        //         return repeat([this, size, off, written, p]() mutable {
        //             auto&& priority_class = service::get_local_commitlog_priority();
        //             return _file.dma_write(off + *written, p + *written, size - *written, priority_class).then_wrapped([this, size, written](future<size_t>&& f) {
        //                 try {
        //                     auto bytes = std::get<0>(f.get());
        //                     *written += bytes;
        //                     _segment_manager->totals.bytes_written += bytes;
        //                     _segment_manager->totals.total_size_on_disk += bytes;
        //                     ++_segment_manager->totals.cycle_count;
        //                     if (*written == size) {
        //                         return make_ready_future<stop_iteration>(stop_iteration::yes);
        //                     }
        //                     // gah, partial write. should always get here with dma chunk sized
        //                     // "bytes", but lets make sure...
        //                     clogger.debug("Partial write {}: {}/{} bytes", *this, *written, size);
        //                     *written = align_down(*written, alignment);
        //                     return make_ready_future<stop_iteration>(stop_iteration::no);
        //                     // TODO: retry/ignore/fail/stop - optional behaviour in origin.
        //                     // we fast-fail the whole commit.
        //                 } catch (...) {
        //                     clogger.error("Failed to persist commits to disk for {}: {}", *this, std::current_exception());
        //                     throw;
        //                 }
        //             });
        //         }).finally([this, buf = std::move(buf), size]() mutable {
        //             _segment_manager->release_buffer(std::move(buf));
        //             _segment_manager->notify_memory_written(size);
        //         });
        // }, [me, flush_after, top, rp] { // lambda instead of bind, so we keep "me" alive.
        //     assert(me->_pending_ops.has_operation(rp));
        //     return flush_after ? me->do_flush(top) : make_ready_future<sseg_ptr>(me);
        // });
        unimplemented!()
    }

    async fn finish_and_get_new(&self) -> Result<Segment> {
        self.inner.borrow_mut().closed = true;
        let self_clone = self.clone();
        spawn(async move {
            self_clone.sync(false).await.map_err(|_| ()).unwrap();
        });
        self.inner.segment_manager.active_segment().await
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

    async fn flush_from_start(&self) -> Result<Segment> {
        self.flush(0).await
    }

    async fn flush(&self, pos: u64) -> Result<Segment> {
        unimplemented!();
    }

    fn clear_buffer_slack(&self) {
        let slack = clear_buffer_slack(&mut self.inner.borrow_mut().buffer);
        self.inner.segment_manager.record_slack(slack);
    }

    fn position(&self) -> Position {
        self.inner.file_pos + self.inner.buffer.len() as u64
    }

    fn size_on_disk(&self) -> u64 {
        self.inner.file_pos
    }
}

fn align_up(buf: &[u8], align: usize) -> usize {
    (buf.len() + align - 1) & !(align - 1)
}

fn clear_buffer_slack(buf: &mut BytesMut) -> usize {
    let new_size = align_up(&buf, ALIGNMENT);
    let slack_size = new_size - buf.len();
    // TODO(jkozlowski): Get rid of this sloppy allocation.
    let slack = vec![0 as u8; slack_size];
    buf.extend_from_slice(&slack[..]);
    slack_size
}

impl Drop for Inner {
    fn drop(&mut self) {
        // TODO(jakubk): Make sure stuff gets closed and deleted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hamcrest2::assert_that;
    use hamcrest2::prelude::*;

    #[test]
    fn test_align_up() {
        assert_that!(align_up(&vec![0;0][0..], ALIGNMENT), is(eq(0)));
        for i in 1..ALIGNMENT {
            assert_that!(align_up(&vec![0;i][0..], ALIGNMENT), is(eq(ALIGNMENT)));
        }
    }

    #[test]
    fn test_clear_buffer_slack() {
        assert_buffer_slack(0, 0);
        for i in 1..ALIGNMENT {
            assert_buffer_slack(i, ALIGNMENT);
        }
    }

    fn assert_buffer_slack(before_len: usize, after_len: usize) {
        let mut bytes = buf(before_len);
        clear_buffer_slack(&mut bytes);
        assert_that!(bytes.len(), is(eq(after_len)));
    }

    fn buf(len: usize) -> BytesMut {
        let mut bytes = BytesMut::new();
        bytes.reserve(len);
        for i in 0..len {
            bytes.put_u8(1);
        }
        assert_that!(bytes.len(), is(eq(len)));
        bytes
    }
}
