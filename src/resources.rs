use std::cmp;
use std::cmp::{Ordering};
use std::vec::{Vec};
use std::collections::{HashMap};

pub type CpuId = u32;

#[derive(Debug)]
pub struct Cpu {
    cpu_id: CpuId,
    mem: Vec<Memory>
}

#[derive(Debug)]
pub struct Memory {
    bytes: usize,
    nodeid: u32,
}

custom_derive! {
    #[derive(Default, Debug, Builder)]
    pub struct Configuration {
        total_memory:    Option<usize>,
        reserve_memory:  Option<usize>,  // if total_memory not specified
        cpus:            Option<usize>,
        cpu_set:         Option<HashSet<CpuId>>,
        max_io_requests: Option<usize>,
        io_queues:       Option<usize>
    }
}

custom_derive! {
    #[derive(Default, Debug, Builder)]
    pub struct Resources {
        cpus: Vec<Cpu>
        //    io_queue_topology io_queues;
    }
}

impl Configuration {

    pub fn get_total_memory(&self) -> Option<usize> {
        return self.total_memory;
    }

    pub fn get_reserve_memory(&self) -> Option<usize> {
        return self.reserve_memory;
    }

    pub fn get_cpus(&self) -> Option<usize> {
        return self.cpus;
    }

    // yuck!
    pub fn build(&self) -> Self {
        return Configuration {
            total_memory:    self.total_memory.clone(),
            reserve_memory:  self.reserve_memory.clone(),
            cpus:            self.cpus.clone(),
            cpu_set:         self.cpu_set.clone(),
            max_io_requests: self.max_io_requests.clone(),
            io_queues:       self.io_queues.clone()
        }
    }
}

const DEFAULT_PANIC_FACTOR: f32 = 1.0;

fn calculate_memory(c: &Configuration, mut available_memory: usize, panic_factor: f32) -> hwloc_error::Result<usize> {
    //  size_t default_reserve_memory = std::max<size_t>(1 << 30, 0.05 * available_memory) * panic_factor;
    let useable_memory: f32 = 0.05f32 * available_memory as f32;
    let default_reserve_memory: usize = (memory_to_reserve(useable_memory) * panic_factor) as usize;

    //  auto reserve = c.reserve_memory.value_or(default_reserve_memory);
    let reserve: usize = c.get_reserve_memory().unwrap_or(default_reserve_memory);

    //  size_t min_memory = 500'000'000;
    let min_memory: usize = 500_000_000;

    //  if (available_memory >= reserve + min_memory) {
    if available_memory >= reserve + min_memory {
        //  available_memory -= reserve;
        available_memory -= reserve;
    } else {
        //  // Allow starting up even in low memory configurations (e.g. 2GB boot2docker VM)
        //  available_memory = min_memory;
        available_memory = min_memory;
    }

    //  size_t mem = c.total_memory.value_or(available_memory);
    let mem: usize = c.get_total_memory().unwrap_or(available_memory);
    //  if (mem > available_memory) {
    if mem > available_memory {
        //  throw std::runtime_error("insufficient physical memory");
        return Err(hwloc_error::ErrorKind::InsufficientPhysicalMemory(mem, available_memory).into());
    } else {
        //  return mem;
        return Ok(mem);
    }
}

fn memory_to_reserve(useable_memory: f32) -> f32 {
    let min_memory: f32 = 1u32.wrapping_shl(30) as f32;
    match min_memory.partial_cmp(&useable_memory).unwrap_or(Ordering::Equal) {
        Ordering::Equal   => min_memory,
        Ordering::Less    => useable_memory,
        Ordering::Greater => min_memory
    }
}

extern crate hwloc;
use error::hwloc_error;

use align;
use std::collections::HashSet;
use hwloc::{Topology, TopologyObject, ObjectType, CpuSet};

const EXPECTED_SIZE_AT_DEPTH: usize = 1;

fn find_memory_depth(topology: &Topology) -> hwloc_error::Result<usize> {
    //  auto depth = hwloc_get_type_depth(topology, HWLOC_OBJ_PU);
    let mut depth: u32 = try!(topology.depth_for_type(&ObjectType::PU));

    //  auto obj = hwloc_get_next_obj_by_depth(topology, depth, nullptr);
    let objs = topology.objects_at_depth(depth);
    let mut obj: Option<&TopologyObject> = objs.first().map(|obj1| *obj1);

    //  while (!obj->memory.local_memory && obj) {
    while obj.is_some() && obj.unwrap().memory().local_memory() == 0 {
        // obj = hwloc_get_ancestor_obj_by_depth(topology, --depth, obj);
        obj = obj.unwrap().parent();
        depth -= 1;
    //  }
    }

    //  assert(obj);
    assert!(obj.is_some());
    //  return depth;
    Ok(depth as usize)
}

//  static size_t alloc_from_node(cpu& this_cpu, hwloc_obj_t node, std::unordered_map<hwloc_obj_t, size_t>& used_mem, size_t alloc) {
fn alloc_from_node(this_cpu: &mut Cpu,
                   node: &TopologyObject,
                   used_mem: &mut HashMap<*const TopologyObject, u64>,
                   alloc: u64) -> usize {
    //  auto taken = std::min(node->memory.local_memory - used_mem[node], alloc);
    let pnode: *const TopologyObject = node;
    let zero = 0;

    //  auto taken = std::min(node->memory.local_memory - used_mem[node], alloc);
    let used_mem_value = used_mem.entry(pnode).or_insert(0);
    let taken = cmp::min(node.memory().local_memory() - *used_mem_value, alloc);

    //  if (taken) {
    if taken > 0 {
        //  used_mem[node] += taken;
        *used_mem_value += taken;

        //  auto node_id = hwloc_bitmap_first(node->nodeset);
        let node_id = node.nodeset().unwrap().first();

        //  assert(node_id != -1);
        assert!(node_id != -1);

        //  this_cpu.mem.push_back({taken, unsigned(node_id)});
        this_cpu.mem.push(Memory { bytes: taken as usize, nodeid: node_id as u32 })
    //  }
    }

    //  return taken;
    taken as usize
//  }
}

//    distribute_objects(hwloc_topology_t& topology, size_t nobjs) : cpu_sets(nobjs), root(hwloc_get_root_obj(topology)) {
fn distribute_objects(topology: &Topology, nobjs: usize) -> Option<Vec<CpuSet>> {

    let root = topology.object_at_root();
    //    #if HWLOC_API_VERSION >= 0x00010900
    //    hwloc_distrib(topology, &root, 1, cpu_sets.data(), cpu_sets.size(), INT_MAX, 0);
    //    #else
    //    hwloc_distribute(topology, root, cpu_sets.data(), cpu_sets.size(), INT_MAX);
    topology.distribute_objects(root, nobjs as u32)
    //    #endif
}

fn get_pu_obj_for_cpu_id<'a>(topology: &'a Topology, cpu_id: CpuId) -> hwloc_error::Result<&'a TopologyObject> {
    let pu_objs = try!(topology.objects_with_type(&ObjectType::PU));
    let pu_obj_with_cpu_id = try!(pu_objs.into_iter()
                                         .find(| &obj | obj.os_index() == cpu_id)
                                         .ok_or("unable to find pu_obj with cpu_id"));
    Ok(pu_obj_with_cpu_id)
}

pub fn allocate(c: Configuration) -> hwloc_error::Result<Resources> {

    //  // Allocate the topology on stack
    //  hwloc_topology_t topology;
    //  // Init the struct
    //  hwloc_topology_init(&topology);
    //  // Defer deallocating the topology correctly
    //  auto free_hwloc = defer([&] { hwloc_topology_destroy(topology); });
    //  // Load the struct with the current topologuy
    //  hwloc_topology_load(topology);
    let topology = Topology::new();

    //  if (c.cpu_set) {
    if let Some(ref cpu_set) = c.cpu_set {
        //  auto bm = hwloc_bitmap_alloc();
        //  auto free_bm = defer([&] { hwloc_bitmap_free(bm); });
        let mut bitmap = CpuSet::new();

        //  for (auto idx : *c.cpu_set) {
        //      hwloc_bitmap_set(bm, idx);
        //  }
        for &idx in cpu_set {
            bitmap.set(idx);
        }

        //  auto r = hwloc_topology_restrict(topology, bm,
        //                                   HWLOC_RESTRICT_FLAG_ADAPT_DISTANCES
        //                                   | HWLOC_RESTRICT_FLAG_ADAPT_MISC
        //                                   | HWLOC_RESTRICT_FLAG_ADAPT_IO);
        //  if (r == -1) {
        //      if (errno == ENOMEM) {
        //          throw std::bad_alloc();
        //      }
        //  if (errno == EINVAL) {
        //      throw std::runtime_error("bad cpuset");
        //  }
        //  abort();
        //  }

    //  }
    }

    //  auto machine_depth = hwloc_get_type_depth(topology, HWLOC_OBJ_MACHINE);
    let machine_depth = try!(topology.depth_for_type(&ObjectType::Machine));

    //  assert(hwloc_get_nbobjs_by_depth(topology, machine_depth) == 1);
    let objects_at_depth = topology.objects_at_depth(machine_depth);
    if objects_at_depth.len() != EXPECTED_SIZE_AT_DEPTH {
        return Err(hwloc_error::ErrorKind::UnexpectedSizeAtDepth(machine_depth, objects_at_depth.len(), EXPECTED_SIZE_AT_DEPTH).into());
    }

    //  auto machine = hwloc_get_obj_by_depth(topology, machine_depth, 0);
    let machine = objects_at_depth[0];

    //  auto available_memory = machine->memory.total_memory;
    let available_memory = machine.memory().total_memory() as usize;

    // // hwloc doesn't account for kernel reserved memory, so set panic_factor = 2
    // size_t mem = calculate_memory(c, available_memory, 2);
    let mem: usize = try!(calculate_memory(&c, available_memory, DEFAULT_PANIC_FACTOR));

    //  unsigned available_procs = hwloc_get_nbobjs_by_type(topology, HWLOC_OBJ_PU);
    let available_procs: usize = try!(topology.objects_with_type(&ObjectType::PU)).len();

    //  unsigned procs = c.cpus.value_or(available_procs);
    let procs: usize = c.get_cpus().unwrap_or(available_procs);

    //  if (procs > available_procs) {
    if procs > available_procs {
        //  throw std::runtime_error("insufficient processing units");
        return Err(hwloc_error::ErrorKind::InsufficientProcessingUnits(procs, available_procs).into());
    }

    //  auto mem_per_proc = align_down<size_t>(mem / procs, 2 << 20);
    let mem_per_proc: usize = align::align_down(mem / procs, (2 as usize).wrapping_shl(20));

    //  resources ret;
    let mut ret = Resources::default();

    //  std::unordered_map<hwloc_obj_t, size_t> topo_used_mem;
    let mut topo_used_mem: HashMap<*const TopologyObject, u64> = HashMap::new();

    //  std::vector<std::pair<cpu, size_t>> remains;
    let mut remains: Vec<(Cpu, usize)> = Vec::new();

    //  size_t remain;
    let mut remain: usize;

    //  unsigned depth = find_memory_depth(topology);
    let depth = try!(find_memory_depth(&topology));

    let cpu_sets: Vec<CpuSet> = try!(distribute_objects(&topology, procs).ok_or("unable to distribute objects"));

    // // Divide local memory to cpus
    // for (auto&& cs : cpu_sets()) {
    for cs in cpu_sets {
        //  auto cpu_id = hwloc_bitmap_first(cs);
        let cpu_id = cs.first();

        //  assert(cpu_id != -1);
        assert!(cpu_id != -1);

        //  auto pu = hwloc_get_pu_obj_by_os_index(topology, cpu_id);
        let pu = try!(get_pu_obj_for_cpu_id(&topology, cpu_id as u32));

        //  auto node = hwloc_get_ancestor_obj_by_depth(topology, depth, pu);
        let node = pu.ancestor_by_depth(depth as u32).unwrap();

        //  cpu this_cpu;
        //  this_cpu.cpu_id = cpu_id;
        let mut this_cpu = Cpu { cpu_id: cpu_id as u32, mem: Vec::new() };

        remain = mem_per_proc - alloc_from_node(&mut this_cpu, node, &mut topo_used_mem, mem_per_proc as u64);

        //  remains.emplace_back(std::move(this_cpu), remain);
        remains.push((this_cpu, remain));
    //  }
    }

    //  // Divide the rest of the memory
    //  for (auto&& r : remains) {
    for (mut this_cpu, mut remain) in remains {
        //  cpu this_cpu;
        //  size_t remain;
        //  std::tie(this_cpu, remain) = r;

        //  auto pu = hwloc_get_pu_obj_by_os_index(topology, this_cpu.cpu_id);
        let pu = try!(get_pu_obj_for_cpu_id(&topology, this_cpu.cpu_id));

        //  auto node = hwloc_get_ancestor_obj_by_depth(topology, depth, pu);
        let node = pu.ancestor_by_depth(depth as u32).unwrap();

        //  auto obj = node;
        let mut obj = Some(node);

        //  while (remain) {
        while remain > 0 {
            //  remain -= alloc_from_node(this_cpu, obj, topo_used_mem, remain);
            remain -= alloc_from_node(&mut this_cpu, node, &mut topo_used_mem, remain as u64);
            //  do {
            loop {
                //  obj = hwloc_get_next_obj_by_depth(topology, depth, obj);
                //  } while (!obj);
            }
            //            if (obj == node)
            //                break;
            //  }
        }
        //  assert(!remain);
        assert!(remain == 0);
        //  ret.cpus.push_back(std::move(this_cpu));
        ret.cpus.push(this_cpu);
        //  }
    }

    //  ret.io_queues = allocate_io_queues(topology, c, ret.cpus);

    //  return ret;
    Ok(ret)
}

//#include "resource.hh"
//#include <unistd.h>
//
//namespace resource {
//
//// Without hwloc, we don't support tuning the number of IO queues. So each CPU gets their.
//static io_queue_topology
//allocate_io_queues(configuration c, std::vector<cpu> cpus) {
//io_queue_topology ret;
//
//unsigned nr_cpus = unsigned(cpus.size());
//unsigned max_io_requests = c.max_io_requests.value_or(128 * nr_cpus);
//
//ret.shard_to_coordinator.resize(nr_cpus);
//ret.coordinators.resize(nr_cpus);
//
//for (unsigned shard = 0; shard < nr_cpus; ++shard) {
//ret.shard_to_coordinator[shard] = shard;
//ret.coordinators[shard].capacity =  std::max(max_io_requests / nr_cpus, 1u);
//ret.coordinators[shard].id = shard;
//}
//return ret;
//}
//
//
//resources allocate(configuration c) {
//resources ret;
//
//auto available_memory = ::sysconf(_SC_PAGESIZE) * size_t(::sysconf(_SC_PHYS_PAGES));
//auto mem = calculate_memory(c, available_memory);
//auto cpuset_procs = c.cpu_set ? c.cpu_set->size() : nr_processing_units();
//auto procs = c.cpus.value_or(cpuset_procs);
//ret.cpus.reserve(procs);
//for (unsigned i = 0; i < procs; ++i) {
//ret.cpus.push_back(cpu{i, {{mem / procs, 0}}});
//}
//
//ret.io_queues = allocate_io_queues(c, ret.cpus);
//return ret;
//}
//
//unsigned nr_processing_units() {
//return ::sysconf(_SC_NPROCESSORS_ONLN);
//}
//
//}