The Road to Seastar
===================

* Custom allocator (memory.hh) that does NUMA configuration, when enabled.
* reactor on each CPU (reactor.hh)
* abstract network stack (net/*), which can either run on normal Posix stack,
  or do kernel bypass (DPDK)
* smp: starts a number of reactors (configurable) and sets up io queues 
  and bidirectional message channels between them.

* Setting affinity: https://github.com/terminalcloud/rust-scheduler, windows: https://github.com/retep998/wio-rs
* Custom allocator: https://doc.rust-lang.org/book/custom-allocators.html
* hwloc bindings: https://github.com/daschl/hwloc-rs; http://nitschinger.at/Discovering-Hardware-Topology-in-Rust/

Notes
-----

* https://github.com/scylladb/seastar/wiki/SMP - Symmetric multiprocessing

file io
-------

Everything basically in file.hh, file-impl.hh, posix.hh, and some in reactor.c.

* posix_file_impl (reactor.c): issues io_prep_preadv call to the engine().submit_io_read.
* Interesting: https://github.com/facebook/folly/tree/master/folly/experimental/io
* reactor::run() is where everything starts proper

app_template
------------

smp
---

called from app-template: 
void smp::configure(boost::program_options::variables_map configuration)

Looks like this is the entry that start the engine: called by the main thread.

# Disable signals (sigdelset)
# Read out config to figure out:
## Thread affinity (whether threads should be pinned to cores, 
   i.e. should the OS be disallowed to migrate threads across CPUs)
## Id of the main thread
## Number of CPUs available
## Number of CPUs to use
## Total memory to use
### Adjust if using DPDK
## Whether to reserve memory
## Hugepages
## Whether to mlock
## max-io-requests
## num-io-queues
# call resource::allocate(rc); to prepare an allocation
## this uses hwloc to allocate memory etc. to the number of cores:
   calls hwloc functions to figure out what is available in terms of CPUs 
   and memory and calculate per CPU allocations.
## Figures out memory depth?
## calls hwloc_distribute (I don't think it actually sets the hierarchy, just 
   figures out the hierarchy.
## calls allocate_io_queues
# Pins the main thread to CPU 0.
# Configures the memory: memory::configure(allocations[0].mem, hugepages_path)
# abort-on-seastar-bad-alloc
# Init dpdk is enabled: dpdk::eal::init
# Prepare thread loops: store them in _threads in case of posix:
  Each thread will basically go through a similar initialisation as the main thread:
  this in fact calls the posix_thread::posix_thread(std::function<void ()> func) (posix.cc)
  so actually starts the threads.
## Affinity
## memory::configure(allocation.mem, hugepages_path);
## Disabling signals
## allocate_reactor()
## Setting thread-local engine id
## Storing it's engine's pointer in _reactors
## Calls alloc_io_queue
## Waits until all reactors are registered: reactors_registered.wait().
## Waits until all queues constructed: smp_queues_constructed.wait().
## start_all_queues();
## assign_io_queue(i, queue_idx);
## Waits until all reactors init: inited.wait();
## engine().configure(configuration);
## Starts the loop: engine().run();
# allocate_reactor();
# alloc_io_queue(0)
# if _using_dpdk, rte_eal_remote_launch(dpdk_thread_adaptor, static_cast<void*>(&*(it++)), i);
# reactors_registered.wait();
# Construct smp queues: pairwise, single-writer/single-reader ring buffers for messages between cores
# smp_queues_constructed.wait();
# start_all_queues();
# assign_io_queue(0, queue_idx);
# inited.wait();
# engine().configure(configuration);
# engine()._lowres_clock = std::make_unique<lowres_clock>();

After that we jump back to app-template:
# Enqueue the initialisation function passed in to run after engine start.
# call auto exit_code = engine().run(); to start the loop on the main thread

reactor
-------

Looks like that is the event loop per core.

* int reactor::run() -> actual loop

memory
------

Custom allocator :(

At the end has the magic stuffz:

```
extern "C"
[[gnu::visibility("default")]]
[[gnu::externally_visible]]
void* malloc(size_t n) throw () {
```

resource
--------

Sets up the hwlock topology.

* resources allocate(configuration c) {

