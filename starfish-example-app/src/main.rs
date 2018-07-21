extern crate spdk_sys as spdk;
extern crate starfish_executor as executor;
extern crate futures;

use std::env;
use std::mem;
use spdk::io_channel;
use spdk::event::AppOpts;
use spdk::bdev;
use spdk::blob_bdev;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let config_file = &args[1];
    let mut opts = AppOpts::new();

    opts.name("hello_blob");
    opts.config_file(config_file.as_str());

    let ret = opts.start(|| {
        let executor = executor::initialize();
        
        // TODO: fixup
        mem::forget(executor);

        // Register the executor poller
        io_channel::poller_register(|| {
            return executor::pure_poll();
        });

        let mut bdev = bdev::get_by_name("Malloc0").expect("bdev not found");
        println!("{:?}", bdev);

        let bs_dev = blob_bdev::create_bs_dev(&mut bdev).expect("could not create bs_dev");
        println!("{:?}", bs_dev);
    });
}

// static int
// bdev_aio_create_cb(void *io_device, void *ctx_buf)
// {
// 	struct bdev_aio_io_channel *ch = ctx_buf;

// 	if (bdev_aio_initialize_io_channel(ch) != 0) {
// 		return -1;
// 	}

// 	ch->poller = spdk_poller_register(bdev_aio_poll, ch, 0);
// 	return 0;
// }

// static int
// bdev_aio_poll(void *arg)
// {
// 	struct bdev_aio_io_channel *ch = arg;
// 	int nr, i;
// 	enum spdk_bdev_io_status status;
// 	struct bdev_aio_task *aio_task;
// 	struct timespec timeout;
// 	struct io_event events[SPDK_AIO_QUEUE_DEPTH];

// 	timeout.tv_sec = 0;
// 	timeout.tv_nsec = 0;

// 	nr = io_getevents(ch->io_ctx, 1, SPDK_AIO_QUEUE_DEPTH,
// 			  events, &timeout);

// 	if (nr < 0) {
// 		SPDK_ERRLOG("%s: io_getevents returned %d\n", __func__, nr);
// 		return -1;
// 	}

// 	for (i = 0; i < nr; i++) {
// 		aio_task = events[i].data;
// 		if (events[i].res != aio_task->len) {
// 			status = SPDK_BDEV_IO_STATUS_FAILED;
// 		} else {
// 			status = SPDK_BDEV_IO_STATUS_SUCCESS;
// 		}

// 		spdk_bdev_io_complete(spdk_bdev_io_from_ctx(aio_task), status);
// 		ch->io_inflight--;
// 	}

// 	return nr;
// }
