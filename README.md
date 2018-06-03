# starfish

Async programming with spdk for rust.

## Running

```
$ HUGEMEM=5120 ./rust-spdk/scripts/setup.sh
$ RUST_LOG=debug cargo build --all-targets
$ RUST_LOG=debug cargo build -p rust-spdk
$ RUST_LOG=debug cargo run -p rust-spdk
$ ls -la /mnt/huge/spdk_*_*
$ rm -rf /mnt/huge/spdk_*_*
$ objdump -g /usr/local/lib/librte_mempool.a
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

* https://doc.rust-lang.org/nightly/rustc/lints/listing/warn-by-default.html#non-upper-case-globals

## Issues

* When starting DPDK, in rte_mempool_opts.c, rte_mempool_ops_table.num_ops is not set to anything,
  so it is not possible to register the created mempool. This is the first failure on the preferred socket. Need to see what the second failure is, but this should succeed on the first try.
* My code does not work on the big machine.
* Running the spdk example that my code is based on fails in the same way :(
* Building the original hello_blob example and running it on a normal machine works:
  * --Works with spdk ./script/setup.sh and mine--
  * --Maybe it's the debug build? Unlikely, I first tried running without. On on the normal machine
    my code is running non-debug build.--
  * Maybe I am not loading all the required static/dynamic libraries? Maybe I am loading the wrong kind of stuff.

Actually, it fails in a slightly more spectacular manner:

```
root@7dbbc383d45d:/tmp/spdk/examples/blob/hello_world# ./hello_blob
hello_blob.c: 449:main: *NOTICE*: entry
Starting SPDK v18.04 / DPDK 18.02.0 initialization...
[ DPDK EAL parameters: hello_blob -c 0x1 --file-prefix=spdk_pid4047 ]
EAL: Detected 2 lcore(s)
EAL: No free hugepages reported in hugepages-1048576kB
EAL: Multi-process socket /var/run/.spdk_pid4047_unix
EAL: Probing VFIO support...
EAL: NUMA support not available consider that all memory is in socket_id 0
app.c: 443:spdk_app_start: *NOTICE*: Total cores available: 1
reactor.c: 650:spdk_reactors_init: *NOTICE*: Occupied cpu socket mask is 0x1
Bus error (core dumped)
```

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

Result of ldd:

```
-bash-4.2# ldd hello_blob
                linux-vdso.so.1 =>  (0x00007ffdc03ee000)
                libaio.so.1 => /lib64/libaio.so.1 (0x00007f42823bf000)
                libnuma.so.1 => /lib64/libnuma.so.1 (0x00007f42821b3000)
                libdl.so.2 => /lib64/libdl.so.2 (0x00007f4281fae000)
                librt.so.1 => /lib64/librt.so.1 (0x00007f4281da6000)
                libuuid.so.1 => /lib64/libuuid.so.1 (0x00007f4281ba1000)
                libpthread.so.0 => /lib64/libpthread.so.0 (0x00007f4281984000)
                libc.so.6 => /lib64/libc.so.6 (0x00007f42815c1000)
                /lib64/ld-linux-x86-64.so.2 (0x00005581e8f23000)
                libgcc_s.so.1 => /lib64/libgcc_s.so.1 (0x00007f42813ab000)
```

Makefile output when linking hello_world:

```
echo "OBJS: hello_blob.o "
OBJS: hello_blob.o
SPDK_LIB_FILES: /tmp/spdk/build/lib/libspdk_event_bdev.a /tmp/spdk/build/lib/libspdk_event_copy.a /tmp/spdk/build/lib/libspdk_blobfs.a /tmp/spdk/build/lib/libspdk_blob.a /tmp/spdk/build/lib/libspdk_bdev.a /tmp/spdk/build/lib/libspdk_blob_bdev.a /tmp/spdk/build/lib/libspdk_copy.a /tmp/spdk/build/lib/libspdk_event.a /tmp/spdk/build/lib/libspdk_util.a /tmp/spdk/build/lib/libspdk_conf.a /tmp/spdk/build/lib/libspdk_trace.a /tmp/spdk/build/lib/libspdk_log.a /tmp/spdk/build/lib/libspdk_jsonrpc.a /tmp/spdk/build/lib/libspdk_json.a /tmp/spdk/build/lib/libspdk_rpc.a
BLOCKDEV_MODULES_FILES: /tmp/spdk/build/lib/libspdk_vbdev_lvol.a /tmp/spdk/build/lib/libspdk_blob.a /tmp/spdk/build/lib/libspdk_blob_bdev.a /tmp/spdk/build/lib/libspdk_lvol.a /tmp/spdk/build/lib/libspdk_bdev_malloc.a /tmp/spdk/build/lib/libspdk_bdev_null.a /tmp/spdk/build/lib/libspdk_bdev_nvme.a /tmp/spdk/build/lib/libspdk_nvme.a /tmp/spdk/build/lib/libspdk_vbdev_passthru.a /tmp/spdk/build/lib/libspdk_vbdev_error.a /tmp/spdk/build/lib/libspdk_vbdev_gpt.a /tmp/spdk/build/lib/libspdk_vbdev_split.a /tmp/spdk/build/lib/libspdk_bdev_aio.a /tmp/spdk/build/lib/libspdk_bdev_virtio.a /tmp/spdk/build/lib/libspdk_virtio.a
LINKER_MODULES:
ENV_LIBS: /tmp/spdk/build/lib/libspdk_env_dpdk.a /tmp/spdk/dpdk/build/lib/librte_eal.a /tmp/spdk/dpdk/build/lib/librte_mempool.a /tmp/spdk/dpdk/build/lib/librte_ring.a /tmp/spdk/dpdk/build/lib/librte_mempool_ring.a /tmp/spdk/dpdk/build/lib/librte_pci.a /tmp/spdk/dpdk/build/lib/librte_bus_pci.a
```
