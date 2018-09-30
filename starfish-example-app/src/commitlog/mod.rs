use crc::{crc64, Hasher64};
use spdk_sys::blob::BlobId;
use futures::Future;
use failure::Error;

// /*
//  * Commit Log tracks every write operation into the system. The aim of
//  * the commit log is to be able to successfully recover data that was
//  * not stored to disk via the Memtable.
//  *
//  * This impl is cassandra log format compatible (for what it is worth).
//  * The behaviour is similar, but not 100% identical as "stock cl".
//  *
//  * Files are managed with "normal" file writes (as normal as seastar
//  * gets) - no mmapping. Data is kept in internal buffers which, when
//  * full, are written to disk (see below). Files are also flushed
//  * periodically (or always), ensuring all data is written + writes are
//  * complete.
//  *
//  * In BATCH mode, every write to the log will also send the data to disk
//  * + issue a flush and wait for both to complete.
//  *
//  * In PERIODIC mode, most writes will only add to the internal memory
//  * buffers. If the mem buffer is saturated, data is sent to disk, but we
//  * don't wait for the write to complete. However, if periodic (timer)
//  * flushing has not been done in X ms, we will write + flush to file. In
//  * which case we wait for it.
//  *
//  * The commitlog does not guarantee any ordering between "add" callers
//  * (due to the above). The actual order in the commitlog is however
//  * identified by the replay_position returned.
//  *
//  * Like the stock cl, the log segments keep track of the highest dirty
//  * (added) internal position for a given table id (cf_id_type / UUID).
//  * Code should ensure to use discard_completed_segments with UUID +
//  * highest rp once a memtable has been flushed. This will allow
//  * discarding used segments. Failure to do so will keep stuff
//  * indefinately.
//  */
struct CommitLog {

}

impl CommitLog {

    pub fn create(config: Config) -> impl Future<Output=Result<CommitLog, Error>> {
        async {
            Ok(CommitLog {})
        }
    }
}

struct SegmentManager {

}

impl SegmentManager {

}

//  * A single commit log file on disk. Manages creation of the file and writing mutations to disk,
//  * as well as tracking the last mutation position of any "dirty" CFs covered by the segment file. Segment
//  * files are initially allocated to a fixed size and can grow to accomidate a larger value if necessary.
//  *
//  * The IO flow is somewhat convoluted and goes something like this:
//  *
//  * Mutation path:
//  *  - Adding data to the segment usually writes into the internal buffer
//  *  - On EOB or overflow we issue a write to disk ("cycle").
//  *      - A cycle call will acquire the segment read lock and send the
//  *        buffer to the corresponding position in the file
//  *  - If we are periodic and crossed a timing threshold, or running "batch" mode
//  *    we might be forced to issue a flush ("sync") after adding data
//  *      - A sync call acquires the write lock, thus locking out writes
//  *        and waiting for pending writes to finish. It then checks the
//  *        high data mark, and issues the actual file flush.
//  *        Note that the write lock is released prior to issuing the
//  *        actual file flush, thus we are allowed to write data to
//  *        after a flush point concurrently with a pending flush.
//  *
//  * Sync timer:
//  *  - In periodic mode, we try to primarily issue sync calls in
//  *    a timer task issued every N seconds. The timer does the same
//  *    operation as the above described sync, and resets the timeout
//  *    so that mutation path will not trigger syncs and delay.
//  *
//  * Note that we do not care which order segment chunks finish writing
//  * to disk, other than all below a flush point must finish before flushing.
//  *
//  * We currently do not wait for flushes to finish before issueing the next
//  * cycle call ("after" flush point in the file). This might not be optimal.
//  *
//  * To close and finish a segment, we first close the gate object that guards
//  * writing data to it, then flush it fully (including waiting for futures create
//  * by the timer to run their course), and finally wait for it to
//  * become "clean", i.e. get notified that all mutations it holds have been
//  * persisted to sstables elsewhere. Once this is done, we can delete the
//  * segment. If a segment (object) is deleted without being fully clean, we
//  * do not remove the file on disk.
//  *
//  */
struct Segment {
    blob_id: BlobId,
    
}

enum SyncMode {
    Batch, Periodic
}

struct Config {
    commitlog_total_space_in_mb: u64,
    commitlog_segment_size_in_mb: u64,
    commitlog_sync_period_in_ms: u64,

    // Max number of segments to keep in pre-alloc reserve.
    // Not (yet) configurable from scylla.conf.
    max_reserve_segments: u64,

    // Max active writes/flushes. Default value
    // zero means try to figure it out ourselves
    max_active_writes: u64,
    max_active_flushes: u64,

    sync_mode: SyncMode,

    // const db::extensions * extensions = nullptr;
}

impl Default for Config {
    fn default() -> Config {
        Config {
            commitlog_total_space_in_mb: 0,
            commitlog_segment_size_in_mb: 64,
            commitlog_sync_period_in_ms: 10 * 1000,
            max_reserve_segments: 12,
            max_active_writes: 0,
            max_active_flushes: 0,
            sync_mode: SyncMode::Periodic,
        }
    }
}

struct Descriptor {
    blob_id: BlobId
}

// let mut digest = crc64::Digest::new(crc64::ECMA);
// digest.write(b"123456789");
// assert_eq!(digest.sum64(), 0x995dc9bbdf1939fa);

#[cfg(test)]
mod tests {

}