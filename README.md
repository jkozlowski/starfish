# starfish

Async programming with spdk for rust.

## Running

```
$ HUGEMEM=5120 ./rust-spdk/scripts/setup.sh
$ RUST_LOG=debug cargo build --all-targets
$ RUST_LOG=debug cargo build -p rust-spdk
$ RUST_LOG=debug cargo run -p rust-spdk
# For now need to run in the directory, for .cargo/config to be picked up
$ cd rust-spdk; cargo run
$ ls -la /mnt/huge/spdk_*_*
$ rm -rf /mnt/huge/spdk_*_*
$ cargo build -vv --release # shows tool args
$ sshfs ec2-user@ec2-18-219-231-112.us-east-2.compute.amazonaws.com:/home/ec2-user/code/starfish ~/Programming/starfish-ec2/
$ sudo umount -f starfish-ec2
```

## SPDK examples

- https://github.com/spdk/spdk/blob/2c7e3a05e3dd68fa4b2e35515e11a03b3c96dc58/lib/rocksdb/env_spdk.cc
- https://github.com/spdk/spdk/blob/cf9e099862ee973b3a0ac4a75da141263c91014b/doc/concurrency.md
- https://github.com/spdk/spdk/blob/28589dbbe864bd035916b8b7e52c20e25de91d31/lib/event/app.c
- https://github.com/spdk/spdk/blob/cc87019ab65be532ad8ae7115c71ce20b6b55824/etc/spdk/vhost.conf.in

## DPDK examples

- https://github.com/scylladb/dpdk/tree/master/examples
- https://dpdk.org/doc/guides/prog_guide/dev_kit_root_make_help.html
- https://dpdk.org/doc/guides/linux_gsg/build_dpdk.html

## Futures and async/await

- https://internals.rust-lang.org/t/explicit-future-construction-implicit-await/7344
- https://internals.rust-lang.org/t/pre-rfc-cps-transform-for-generators/7120

## Useful

- https://github.com/japaric/xargo/issues/45
- https://github.com/hnes/libaco

### Format and fix lints

```
$ cargo clippy --all -- -Dwarnings
$ cargo +nightly fmt --all
```

### Pretty print macros:

```
$ cargo rustc -- --pretty=expanded -Z unstable-options
```

### Print link args:

```
$ rustc --bin hello_world -- -Z print-link-args
```

### Lints

- `https://doc.rust-lang.org/nightly/rustc/lints/listing/warn-by-default.html#non-upper-case-globals`
