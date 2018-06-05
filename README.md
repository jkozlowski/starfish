# starfish

Async programming with spdk for rust.

## Running

```
$ HUGEMEM=5120 ./rust-spdk/scripts/setup.sh
$ RUST_LOG=debug cargo build --all-targets
$ RUST_LOG=debug cargo build -p rust-spdk
$ RUST_LOG=debug cargo run -p rust-spdk
# For now need ot run in the directory, for .cargo/config to be picked up
$ cd rust-spdk; cargo run
$ ls -la /mnt/huge/spdk_*_*
$ rm -rf /mnt/huge/spdk_*_*
```

## SPDK examples

* https://github.com/spdk/spdk/blob/2c7e3a05e3dd68fa4b2e35515e11a03b3c96dc58/lib/rocksdb/env_spdk.cc
* https://github.com/spdk/spdk/blob/cf9e099862ee973b3a0ac4a75da141263c91014b/doc/concurrency.md
* https://github.com/spdk/spdk/blob/28589dbbe864bd035916b8b7e52c20e25de91d31/lib/event/app.c
* https://github.com/spdk/spdk/blob/cc87019ab65be532ad8ae7115c71ce20b6b55824/etc/spdk/vhost.conf.in

## DPDK examples

* https://github.com/scylladb/dpdk/tree/master/examples
* https://dpdk.org/doc/guides/prog_guide/dev_kit_root_make_help.html
* https://dpdk.org/doc/guides/linux_gsg/build_dpdk.html

## Useful

Pretty print macros:

```
$ cargo rustc -- --pretty=expanded -Z unstable-options
```

Lints:

* https://doc.rust-lang.org/nightly/rustc/lints/listing/warn-by-default.html#non-upper-case-globals`

## Successful run

```
-bash-4.2# ./hello_blob
hello_blob.c: 449:main: *NOTICE*: entry
Starting SPDK v18.07-pre / DPDK 18.02.0 initialization...
[ DPDK EAL parameters: hello_blob -c 0x1 --file-prefix=spdk_pid3422 ]
EAL: Detected 2 lcore(s)
EAL: No free hugepages reported in hugepages-1048576kB
EAL: Multi-process socket /var/run/.spdk_pid3422_unix
EAL: Probing VFIO support...
app.c: 521:spdk_app_start: *NOTICE*: Total cores available: 1
reactor.c: 669:spdk_reactors_init: *NOTICE*: Occupied cpu socket mask is 0x1
reactor.c: 453:_spdk_reactor_run: *NOTICE*: Reactor started on core 0 on socket 0
hello_blob.c: 405:hello_start: *NOTICE*: entry
hello_blob.c: 370:bs_init_complete: *NOTICE*: entry
hello_blob.c: 378:bs_init_complete: *NOTICE*: blobstore: 0x1612660
hello_blob.c: 357:create_blob: *NOTICE*: entry
hello_blob.c: 336:blob_create_complete: *NOTICE*: entry
hello_blob.c: 344:blob_create_complete: *NOTICE*: new blob id 4294967296
hello_blob.c: 305:open_complete: *NOTICE*: entry
hello_blob.c: 316:open_complete: *NOTICE*: blobstore has FREE clusters of 15
hello_blob.c: 282:resize_complete: *NOTICE*: resized blob now has USED clusters of 15
hello_blob.c: 258:sync_complete: *NOTICE*: entry
hello_blob.c: 221:blob_write: *NOTICE*: entry
hello_blob.c: 204:write_complete: *NOTICE*: entry
hello_blob.c: 180:read_blob: *NOTICE*: entry
hello_blob.c: 153:read_complete: *NOTICE*: entry
hello_blob.c: 167:read_complete: *NOTICE*: read SUCCESS and data matches!
hello_blob.c: 133:delete_blob: *NOTICE*: entry
hello_blob.c: 114:delete_complete: *NOTICE*: entry
hello_blob.c:  77:unload_complete: *NOTICE*: entry
hello_blob.c: 485:main: *NOTICE*: SUCCCESS!
```
