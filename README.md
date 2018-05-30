# tokio-smp

```
$ RUST_LOG=debug cargo build -p smp-spdk
$ RUST_LOG=debug cargo run -p smp-spdk
```

## Useful

Pretty print macros:

```
$ cargo rustc -- --pretty=expanded -Z unstable-options
```

Lints:

* https://doc.rust-lang.org/nightly/rustc/lints/listing/warn-by-default.html#non-upper-case-globals
