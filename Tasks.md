### Get a development environment on the VM

- Need a way to edit files and be immediately synced to the VM for execution.
- Use sshfs.

### Compile and run on the VM

- Currently having problems with linking, I cannot figure out how to get it to link correctly against dpdk and spdk libraries.
- It is hard to enable the special linked flags just for the library code; they seem to apply to everything that crate uses.
- I need to get this working quickly, because my interest is wearing off and it is very tedious: I keep finding excuses to not work on this and I am visibly tired and frustrated with this. I need to very carefully examine and learn what I need to figure out these linking issues.

So:

- What is the linker config that is applied in the docker build?
- Can .cargo/config be used along with build.rs; does one of the take precedence?
- Shall I simply generate code in build.rs, along with .cargo/config, to make sure I can use loops and whatnot and generate the right config based on the environment I am in; kinda like ./configure script.
- Why is it suddently not compiling properly in Docker?
- Alternative: get the exact set of linked commands used in the C example and somehow replicate them.
- https://doc.rust-lang.org/cargo/reference/build-scripts.html
- Could it be that I first managed to mostly get things working with ./build.rs but then started overriding with .cargo/config, but some of the opts from build.rs stuck. But when rerunning they do not seem to stick?
- Maybe I can learn from https://bitbucket.org/tari/llvm-sys.rs?
- Can I learn something from https://github.com/nvfuse/nvfuse?
- Also useful stuff here: https://github.com/topics/spdk
- Maybe useful stuff here: https://github.com/alexcrichton/complicated-linkage-example
- And here: https://internals.rust-lang.org/t/perfecting-rust-packaging-the-plan/2767
- https://doc.rust-lang.org/cargo/reference/manifest.html
- https://kazlauskas.me/entries/writing-proper-buildrs-scripts.html
- https://rust-lang-nursery.github.io/rust-cookbook/build_tools.html

Plan:

- Create a minimal Makefile that will include spdk and dpdk makefiles and steal their flags.
- Those flags are then somehow passed to my rust build.
- Rust build publishes those along: how do I translate? Can I just pass through?
- Could I use some tricks with wrapping functions in my own c files?
- Could I create a c wrapper that force-links everything together in the right order?

- Steal from https://crates.io/keywords/ffi and https://crates.io/keywords/bindings.

```
cargo:warn=CPPFLAGS=""""""
LDFLAGS="""-Wl,-z,relro,-z,now -Wl,-z,noexecstack -pthread"""
OBJS=""" """
LIBS="""-Wl,--whole-archive -lspdk_copy_ioat -lspdk_ioat -Wl,--no-whole-archive  -Wl,--whole-archive -lspdk_vbdev_lvol -lspdk_blob -lspdk_blob_bdev -lspdk_lvol -lspdk_bdev_malloc -lspdk_bdev_null -lspdk_bdev_nvme -lspdk_nvme -lspdk_vbdev_passthru -lspdk_vbdev_error -lspdk_vbdev_gpt -lspdk_vbdev_split -lspdk_bdev_aio -lspdk_bdev_virtio -lspdk_virtio -Wl,--no-whole-archive -laio -L/tmp/spdk/build/lib -Wl,--whole-archive -lspdk_event_bdev -lspdk_event_copy -Wl,--no-whole-archive -lspdk_blobfs -lspdk_blob -lspdk_bdev -lspdk_blob_bdev -lspdk_copy -lspdk_event -lspdk_util -lspdk_conf -lspdk_trace -lspdk_log -lspdk_jsonrpc -lspdk_json -lspdk_rpc /tmp/spdk/build/lib/libspdk_env_dpdk.a -Wl,--start-group -Wl,--whole-archive /tmp/spdk/dpdk/build/lib/librte_eal.a /tmp/spdk/dpdk/build/lib/librte_mempool.a /tmp/spdk/dpdk/build/lib/librte_ring.a /tmp/spdk/dpdk/build/lib/librte_mempool_ring.a /tmp/spdk/dpdk/build/lib/librte_pci.a /tmp/spdk/dpdk/build/lib/librte_bus_pci.a -Wl,--end-group -Wl,--no-whole-archive -lnuma -ldl """
SYS_LIBS="""-lrt -luuid"""
```

IL is on fffca7b27dc360be8f9f368733c2523378dd8ccc „Force linking of rte static libs.” With tiny adjustments.

-bash-4.2# cargo --version
cargo 1.26.0 (0e7c5a931 2018-04-06)

-bash-4.2# rustc --version
rustc 1.26.1 (827013a31 2018-05-25)

Running `rustc --crate-name build_script_build rust-spdk/build.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 --cfg 'feature="bla"' --cfg 'feature="default"' -C metadata=deb4a6670d3d6171 -C extra-filename=-deb4a6670d3d6171 --out-dir /tmp/tokio-smp/target/debug/build/rust-spdk-deb4a6670d3d6171 -C incremental=/tmp/tokio-smp/target/debug/incremental -L dependency=/tmp/tokio-smp/target/debug/deps --extern bindgen=/tmp/tokio-smp/target/debug/deps/libbindgen-eac66be723dd0997.rlib -C link-arg=-Wl,--start-group -C link-arg=-Wl,--whole-archive -C link-arg=/tmp/spdk/dpdk/build/lib/librte_eal.a -C link-arg=/tmp/spdk/dpdk/build/lib/librte_mempool.a -C link-arg=/tmp/spdk/dpdk/build/lib/librte_ring.a -C link-arg=/tmp/spdk/dpdk/build/lib/librte_mempool_ring.a -C link-arg=/tmp/spdk/dpdk/build/lib/librte_pci.a -C link-arg=/tmp/spdk/dpdk/build/lib/librte_bus_pci.a -C link-arg=-Wl,--end-group -C link-arg=-Wl,--no-whole-archive -C link-arg=-lnuma -L native=/tmp/tokio-smp/target/debug/build/libloading-c898ce2cf9733652/out`

Then:
Running `rustc --crate-name hello_world rust-spdk/src/hello_world.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 --cfg 'feature="bla"' --cfg 'feature="default"' -C metadata=60e169a2678f4700 -C extra-filename=-60e169a2678f4700 --out-dir /tmp/tokio-smp/target/debug/deps -C incremental=/tmp/tokio-smp/target/debug/incremental -L dependency=/tmp/tokio-smp/target/debug/deps --extern rust_spdk=/tmp/tokio-smp/target/debug/deps/librust_spdk-d29ce9dd73e91611.rlib -C link-arg=-Wl,--start-group -C link-arg=-Wl,--whole-archive -C link-arg=/tmp/spdk/dpdk/build/lib/librte_eal.a -C link-arg=/tmp/spdk/dpdk/build/lib/librte_mempool.a -C link-arg=/tmp/spdk/dpdk/build/lib/librte_ring.a -C link-arg=/tmp/spdk/dpdk/build/lib/librte_mempool_ring.a -C link-arg=/tmp/spdk/dpdk/build/lib/librte_pci.a -C link-arg=/tmp/spdk/dpdk/build/lib/librte_bus_pci.a -C link-arg=-Wl,--end-group -C link-arg=-Wl,--no-whole-archive -C link-arg=-lnuma -L native=/usr/local/lib -L native=/usr/lib/x86_64-linux-gnu`

Suddently getting (red herring):
error: toolchain 'stable-x86_64-unknown-linux-gnu' does not have the binary `rustfmt`
Custom { kind: Other, error: StringError("Internal rustfmt error") }

Same as on docker.

Maybe it is the features that make it work? No, features seem to be disabled.

Now, what is going to happen if I run cargo clean and cargo build -> All deps get rebuilt with the extra flags, same as in docker. I think I somehow managed to apply them just to the library by luck, probably because I made changes to .cargo/config after the deps have bene compiled. However, it looks like on this machine compiling with the extra flags is FINE. SO WHERE IS THE SYS LINKING COMING IN???

= note: /tmp/spdk/dpdk/build/lib/librte_eal.a(eal.o): In function `rte_eal_check_module': /tmp/spdk/dpdk/lib/librte_eal/linuxapp/eal/eal.c:1031: undefined reference to`stat'
/tmp/spdk/dpdk/lib/librte_eal/linuxapp/eal/eal.c:1044: undefined reference to `stat' /tmp/spdk/dpdk/build/lib/librte_eal.a(eal_common_options.o): In function`eal_plugindir_init':
/tmp/spdk/dpdk/lib/librte_eal/common/eal_common_options.c:241: undefined reference to `stat' /tmp/spdk/dpdk/build/lib/librte_eal.a(eal_common_options.o): In function`eal_plugins_init':
/tmp/spdk/dpdk/lib/librte_eal/common/eal_common_options.c:259: undefined reference to `stat' /tmp/spdk/dpdk/lib/librte_eal/common/eal_common_options.c:265: undefined reference to`stat'
/tmp/spdk/dpdk/build/lib/librte_bus_pci.a(pci_uio.o): In function `pci_mknod_uio_dev': /tmp/spdk/dpdk/drivers/bus/pci/linux/pci_uio.c:108: undefined reference to`mknod'
collect2: error: ld returned 1 exit status

https://www.redhat.com/archives/pam-list/1999-February/msg00082.html
https://gcc.gnu.org/ml/gcc-help/1999-11n/msg00456.html
