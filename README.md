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

## Useful

Pretty print macros:

```
$ cargo rustc -- --pretty=expanded -Z unstable-options
```

Lints:

* https://doc.rust-lang.org/nightly/rustc/lints/listing/warn-by-default.html#non-upper-case-globals
