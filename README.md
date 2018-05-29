# tokio-smp

```
$ git submodule update --init --recursive
$ cd smp-spdk/spdk
$ ./configure
$ make install
$ cargo build -p smp-spdk
$ cargo run -p smp-spdk
```

## Useful

Pretty print macros:

```
$ cargo rustc -- --pretty=expanded -Z unstable-options
```

Lints:

* https://doc.rust-lang.org/nightly/rustc/lints/listing/warn-by-default.html#non-upper-case-globals
