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
