# starfish

[![CircleCI](https://circleci.com/gh/jkozlowski/starfish.svg?style=svg)](https://circleci.com/gh/jkozlowski/starfish)

Async programming with spdk for rust (Linux only!).

## Building and Running

```
# First need to build spdk
$ cd /tmp
$ git clone git@github.com:spdk/spdk.git

$ cd /tmp/spdk
$ git checkout v18.07.1
$ git submodule update --init
$ sudo ./scripts/pkgdep.sh

$ ./configure
$ sudo make install
$ ./scripts/setup.sh

# Used for aio backed testing
$ dd if=/dev/zero of=/tmp/aiofile bs=2048 count=5000

$ sudo ldconfig /usr/local/lib

# Need to run dpdk applications as root :(
$ cargo build && sudo target/debug/starfish-example-app starfish-example-app/config/hello_blob.conf
```

## Example apps

* https://github.com/percona/tokudb-engine
* https://github.com/percona/tokudb-engine/wiki/Write-optimized-fractal-tree-storage
* https://github.com/Tokutek
* https://www.percona.com/blog/wp-content/uploads/2011/11/how-fractal-trees-work.pdf

## Futures and async/await

- https://internals.rust-lang.org/t/explicit-future-construction-implicit-await/7344
- https://internals.rust-lang.org/t/pre-rfc-cps-transform-for-generators/7120

## Useful

- https://github.com/japaric/xargo/issues/45
- https://github.com/hnes/libaco
- http://www.f-stack.org/
