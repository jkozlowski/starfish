tokio-smp
=========

Plan:
* Build simple smp: startup a bunch of threads and setup queues between them.
  Once they have setup queues, make them ping each other with messages.


Useful
------

Pretty print macros:

```
$ cargo rustc -- --pretty=expanded -Z unstable-options
```

tokio-file-aio
--------------

Useful
------

* https://www.clever-cloud.com/blog/engineering/2017/03/28/lapin-new-rust-amqp-library/
* https://github.com/slog-rs/misc/tree/2.x.y

Problems with linux signals:
* https://github.com/alexcrichton/tokio-signal/blob/master/src/unix.rs
* https://github.com/carllerche/mio/issues/16
* https://gabrbedd.wordpress.com/2013/07/29/handling-signals-with-signalfd/
* http://lkml.iu.edu/hypermail/linux/kernel/0707.2/1141.html

* https://github.com/diwic/fdringbuf-rs
* https://github.com/polyfractal/bounded-spsc-queue
* https://github.com/fnichol/docker-rust
* https://github.com/carllerche/minidb
* https://github.com/tailhook/tokio-tutorial
* http://fitzgeraldnick.com/2016/12/14/using-libbindgen-in-build-rs.html
* https://github.com/nrc/zero/blob/master/src/lib.rs
* https://github.com/briansmith/untrusted
** https://github.com/briansmith/webpki/blob/master/src/cert.rs
* https://github.com/seanmonstar/httparse/blob/master/src/lib.rs

* https://github.com/servo/heapsize
* https://crates.io/crates/heapsize_derive/

* Strings in Kafka protocol are UTF8
* https://github.com/brson/error-chain/blob/master/examples/all.rs

```
$ RUST_BACKTRACE=1 RUST_LOG=kafka=info cargo run --release kafka
$ RUST_BACKTRACE=1 cargo test --release -- --nocapture
$ echo "Hello" | cargo run --example console-producer
$ cargo expand --release --bin kafka
```

Plan
----

* -temporary_buffer (aligned buffers)-
* Shitty transport that just does ProduceMessage
* Pass that through to the logs and implement the writing and batching of writing
* Implement replaying of logs etc.
* Implement getting of logs
* Start implementing the distributedness
* Implement smp

scylladb:

transport/server.cc
-> cql_server::listen
 -> cql_server::do_accepts
  -> cql_server::connection::process
    -> cql_server::connection::process_request
      -> cql_server::connection::read_frame
        -> cql_server::connection::parse_frame
      -> cql_server::connection::read_and_decompress_frame
      -> cql_server::connection::process_request_one
      -> cql_server::connection::process_query

https://beachape.com/blog/2016/11/02/rust-performance-testing-on-travis-ci/

tokio-proto
-----------

* https://github.com/carllerche/rustbelt-tutorial
* https://github.com/sinkuu/tokio-framecodecs
* https://github.com/google/tarpc
* https://github.com/bfrog/tokio-cql
* https://github.com/carllerche/futures-ext
* https://github.com/carllerche/buffoon
* https://github.com/pingcap/rust-prometheus

http://techblog.cloudperf.net/2016/09/exploring-numa-on-amazon-cloud-instances.html

kafka
-----

ISR - In-Sync Replicas?
LEO - Log-End Offset
https://cwiki.apache.org/confluence/display/KAFKA/kafka+Detailed+Replication+Design+V3
https://engineering.linkedin.com/kafka/benchmarking-apache-kafka-2-million-writes-second-three-cheap-machines
https://medium.com/the-hoard/how-kafkas-storage-internals-work-3a29b02e026#.6k6nc5a9z

* Rust Client: https://github.com/spicavigo/kafka-rust

Write path:
* KafkaRequestHandler#run: Just a parsing routine
* KafkaApis#handle: huge switch statement on the type of message
* KafkaApis#handleProducerRequest: Actual handling of the request
  - Check the producer is authorized to write
  - If yes, prepare a callback to run once everything is replicated (very interesting in itself)
  - call ReplicaManager#appendMessages
* ReplicaManager#appendMessages
  - Append messages to leader replicas of the partition, and wait for them to be replicated to other replicas;
    the callback function will be triggered either when timeout or the required acks are satisfied.
  - Basically ensure that replication happens: first writes locally, then pushes to replicas.
* ReplicaManager#appendToLocalLog: Map[TopicPartition, MessageSet] -> Map[TopicPartition, LogAppendResult]
  - Figure out the partition:
    ``` val partitionOpt = getPartition(topicPartition.topic, topicPartition.partition)```
  - Send the writes to leader:
    ```Partition#appendMessagesToLeader```
  - Update stats
* Partition#appendMessagesToLeader: ByteBufferMessageSet->LogAppendInfo
  - Lookup the Replica for the partition and check that this node is the leader of it
  - If so, check that there is enough in-sync replicas for the write to be safe.
  - Call Log#append
  - Notify ReplicaManager#tryCompleteDelayedFetch
  - Maybe increment HighWatermark: Partition#maybeIncrementLeaderHW
* Log#append
* LogSegment
* OffsetIndex
* TimeIndex

Read Path:
*

Replay:
*

Running
-------

Need Linux server in VirtualBox.

```
$ sudo apt-get update
$ sudo apt-get upgrade
$ sudo apt-get install openssh-server
```

From host:
```
$ VBoxManage modifyvm aio --natpf1 "ssh,tcp,,3022,,22"
$ ssh -p 3022 jakubkozlowski@localhost
Now we're in guest
$ sudo apt-get install virtualbox-guest-utils
$ sudo usermod -aG vboxsf $(whoami)
Restart guest, mount the code directory as "tokio-file-aio" and ssh again
Code should be under /media/sf_tokio-file-aio
$ curl https://sh.rustup.rs -sSf | sh
$ sudo apt-get -q -y install libaio-dev libclang-dev clang pkg-config xfslibs-dev
## Need to use nightly until https://github.com/rust-lang/cargo/pull/3118 makes it to stable; virtualbox doesn't support hard links
$ rustup toolchain install nightly
$ rustup default nightly
$ sudo apt install gdb
$ sudo apt install lldb
$ sudo apt-get install libssl-dev
$ sudo apt-get install libsnappy-dev
$ sudo apt-get install automake
$ sudo apt-get install libtool
$ sudo apt-get install build-essential
$ cargo install cargo-expand
```

Running in Docker
-----------------

```
$ docker build . -t "alpine-bash:latest"
$ docker run -it --rm -p 127.0.0.1:8080:8080 -v $(pwd):/source "alpine-bash:latest" bash
$ export RUST_BACKTRACE=1; cargo build --verbose
$ export RUST_BACKTRACE=1; RUST_LOG=tokio_file_aio=debug,tokio_core=debug cargo run --example read_file
$ cargo rustc -- --pretty=expanded -Z unstable-options
$ ulimit -c unlimited
$ echo '/tmp/core.%e.%p.%t' | sudo tee /proc/sys/kernel/core_pattern
$ RUST_BACKTRACE=1; RUST_LOG=tokio_file_aio=off,tokio_core=trace,read_file=info ./target/debug/examples/read_file
$ lldb target/debug/examples/read_file -c /tmp/core.read_file.13651.1477673455
```

Notes
-----

https://bugs.debian.org/cgi-bin/bugreport.cgi?att=1;bug=418048;filename=eventfd-aio-test.c;msg=10

Required implementations of:
* io_queue (reactor.h, reactor.c)
* thread_pool (reactor.h, reactor.c)
* file
* fair_queue (not 100% necessary straight away, we can probably do away without priority classes).
* semaphore
* gate
* future<io_event> reactor::submit_io
* semaphore _io_context_available
* understand _aio_eventfd

How it works:
* future<io_event> reactor::submit_io(Func prepare_io) (reactor.cc):
  puts things onto _pending_aio; calls flush_pending_aio() if queue full
* bool reactor::flush_pending_aio() (reactor.cc):
  prepares max_aio number of iocb structs and calls io_submit on _io_context
  and those iocbs;
* bool reactor::process_io():
  calls io_getevents and resolves the promises
* process_io is invoked by reactor::io_pollfn

Things left to understand:
* How do you know when to read?

_aio_eventfd:
* Created in void reactor::configure(boost::program_options::variables_map vm):
  looks like the actual creation is in file_desc eventfd(unsigned initval, int flags)
  (file_desc::eventfd(0, 0) calls eventfd()) and then that's wrapped in file_desc and in pollable_fd.
  pollable_fd associates pollable_fd_state with it;

* Started in int reactor::run() -> void reactor::start_aio_eventfd_loop()
* start_aio_eventfd_loop just loops reading from _aio_eventfd when it's ready
* Is the secret in pollable_fd?
* I know now! reactor::start_aio_eventfd_loop calls _aio_eventfd->readable()
  engine().readable(*_s) which eventually calls reactor_backend_epoll::get_epoll_future(fd, &pollable_fd_state::pollin, EPOLLIN);
  which calls epoll_ctl.



https://github.com/jakm/twisted-linux-aio

bool reactor::process_io()
void reactor::configure(boost::program_options::variables_map vm)

_aio_eventfd = pollable_fd(file_desc::eventfd(0, 0));



Plan:

## need to understand eventfd.

need to understand thread_pool
------------------------------

* Has a queue of stuff to do (syscall_work_queue).
* There is a worker thread that that is associated with thread pool (so kinda single-thread thread-pool).
* Worker thread runs in a loop running void thread_pool::work()
* It reads an even from inter_thread_wq._start_eventfd.get_read_fd() and executes it.
* Submissions:
** future<file> reactor::open_file_dma(sstring name, open_flags flags, file_open_options options)
** subscription<directory_entry> posix_file_impl::list_directory(std::function<future<> (directory_entry de)> next)
** future< > reactor::remove_file(sstring pathname)
** future< > reactor::rename_file(sstring old_pathname, sstring new_pathname)
** future< > reactor::link_file(sstring oldpath, sstring newpath)
** future<std::experimental::optional<directory_entry_type>> reactor::file_type(sstring name)
** future<uint64_t> reactor::file_size(sstring pathname)
** future<bool> reactor::file_exists(sstring pathname)
** future<fs_type> reactor::file_system_at(sstring pathname)
** future<file> reactor::open_directory(sstring name)
** future< > reactor::make_directory(sstring name)
** future< > reactor::touch_directory(sstring name)
** future< > posix_file_impl::flush(void)
** future<struct stat> posix_file_impl::stat(void)
** future< > posix_file_impl::truncate(uint64_t length)
** future< > posix_file_impl::discard(uint64_t offset, uint64_t length)
** future< > blockdev_file_impl::discard(uint64_t offset, uint64_t length)
** future< > posix_file_impl::close() noexcept
** future<uint64_t> blockdev_file_impl::size(void)
**



