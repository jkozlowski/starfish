### Running DPDK in Docker

- https://github.com/redhat-performance/docker-dpdk
- https://www.slideshare.net/MichelleHolley1/dpdk-in-containers-handson-lab
- https://software.intel.com/en-us/articles/using-docker-containers-with-open-vswitch-and-dpdk-on-ubuntu-1710
- https://github.com/Eideticom/docker-noload

### Cannot build the thing

- https://github.com/alexcrichton/pkg-config-rs/blob/master/src/lib.rs#L487
- Can I stop at the linking stage and link things with a custom script?
- https://os.phil-opp.com/freestanding-rust-binary/ (cargo rustc -- -Z pre-link-arg=-nostartfiles)
- https://doc.rust-lang.org/1.5.0/book/rust-inside-other-languages.html
- https://www.mankier.com/1/cargo-rustc
- compiler versions?
- https://csclub.uwaterloo.ca/~tbelaire/blog/posts/gba-rust-2.html
- https://blog.filippo.io/rustgo/

### Plan

- Generate and commit bindgen bindings.
- Manually compile those into an object file.
- Link those together with the other libs into a static library.
- Include all this in the other projects, where there is a higher level binding developed.
