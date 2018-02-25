use resources_error;
use libc::{sysconf, _SC_NPROCESSORS_ONLN};
use resources::Resources;
use resources::ResourcesBuilder;
use resources::Configuration;
use resources::Cpu;
use resources::CpuBuilder;
use resources::Memory;
use resources::MemoryBuilder;
use resources::IoQueueTopology;
use resources::IoQueue;
use resources::IoQueueBuilder;
use calculate_memory_default_panic_factor;
use std::cmp::max;

pub fn allocate(c: Configuration) -> resources_error::Result<Resources> {
    //  resources ret;
    let mut ret = ResourcesBuilder::default();

    //  auto available_memory = ::sysconf(_SC_PAGESIZE) * size_t(::sysconf(_SC_PHYS_PAGES));
    let available_memory = try!(get_available_memory());

    //  auto mem = calculate_memory(c, available_memory);
    let mem = try!(calculate_memory_default_panic_factor(&c, available_memory));

    //  auto cpuset_procs = c.cpu_set ? c.cpu_set->size() : nr_processing_units();
    let cpuset_procs = match c.get_cpu_set() {
        Some(cpu_set) => cpu_set.len(),
        None => try!(nr_processing_units()),
    };

    //  auto procs = c.cpus.value_or(cpuset_procs);
    let procs = c.get_cpus().unwrap_or(cpuset_procs);

    //  ret.cpus.reserve(procs);
    let mut cpus = Vec::with_capacity(procs);

    //  for (unsigned i = 0; i < procs; ++i) {
    for cpu_id in 0..procs {
        //  ret.cpus.push_back(cpu{i, {{mem / procs, 0}}});
        let mem = MemoryBuilder::default()
            .bytes(mem / procs)
            .nodeid(0 as u32)
            .build()
            .unwrap();
        let cpu = CpuBuilder::default()
            .cpu_id(cpu_id as u32)
            .mem(vec![mem])
            .build()
            .unwrap();
        cpus.push(cpu);
    }

    //  ret.io_queues = allocate_io_queues(c, ret.cpus);
    let io_queues = allocate_io_queues(&c, &cpus);
    ret.io_queues(io_queues);

    ret.cpus(cpus);

    //  return ret;
    Ok(ret.build().unwrap())
}

#[cfg(target_os = "macos")]
fn get_available_memory() -> resources_error::Result<usize> {
    let mut mem: uint64_t = 0;
    let pmem: *mut uint64_t = &mut mem as *mut _ as *mut uint64_t;
    let ret = unsafe { memsize(pmem) };

    if ret >= 0 {
        Ok(mem as usize)
    } else {
        Err(format!("Error retrieving available memory: {}", ret).into())
    }
}

#[cfg(target_os = "macos")]
extern "C" {
    fn memsize(bytes: *mut uint64_t) -> c_int;
}

#[cfg(target_os = "linux")]
fn get_available_memory() -> resources_error::Result<usize> {
    Ok(1)
}

//// Without hwloc, we don't support tuning the number of IO queues. So each CPU gets theirs.
fn allocate_io_queues(c: &Configuration, cpus: &Vec<Cpu>) -> IoQueueTopology {
    //  io_queue_topology ret;
    let mut ret = IoQueueTopology::default();

    //  unsigned nr_cpus = unsigned(cpus.size());
    let nr_cpus = cpus.len() as usize;

    //  unsigned max_io_requests = c.max_io_requests.value_or(128 * nr_cpus);
    let max_io_requests = c.get_max_io_requests().unwrap_or(128 * nr_cpus);

    //  ret.shard_to_coordinator.resize(nr_cpus);
    ret.get_shard_to_coordinator_mut().reserve(nr_cpus);

    //  ret.coordinators.resize(nr_cpus);
    ret.get_coordinators_mut().reserve(nr_cpus);

    //  for (unsigned shard = 0; shard < nr_cpus; ++shard) {
    for shard in 0..nr_cpus {
        //  ret.shard_to_coordinator[shard] = shard;
        ret.get_shard_to_coordinator_mut().push(shard);

        //  ret.coordinators[shard].id = shard;
        //  ret.coordinators[shard].capacity =  std::max(max_io_requests / nr_cpus, 1u);
        let capacity = max(max_io_requests / nr_cpus, 1);
        let io_queue = IoQueueBuilder::default()
            .id(shard)
            .capacity(capacity)
            .build()
            .unwrap();
        ret.get_coordinators_mut().push(io_queue);
    }

    //  return ret;
    ret
}

fn nr_processing_units() -> resources_error::Result<usize> {
    //  return ::sysconf(_SC_NPROCESSORS_ONLN);
    let ret = unsafe { sysconf(_SC_NPROCESSORS_ONLN) };
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(format!("Unable to get number of processing units: {}", ret).into())
    }
}
