//extern crate hwloc;
//use error::hwloc_error;
//use resources::{calculate_memory, Configuration};
//
//use align;
//use std::collections::HashSet;
//use hwloc::{Topology, TopologyObject, ObjectType, CpuSet};
//
//const EXPECTED_SIZE_AT_DEPTH: usize = 1;
//
//fn find_memory_depth(topology: &Topology) -> hwloc_error::Result<usize> {
//    //  auto depth = hwloc_get_type_depth(topology, HWLOC_OBJ_PU);
//    let mut depth: u32 = try!(topology.depth_for_type(&ObjectType::PU));
//
//    //  auto obj = hwloc_get_next_obj_by_depth(topology, depth, nullptr);
//    let objs = topology.objects_at_depth(depth);
//    let mut obj: Option<&TopologyObject> = objs.first().map(|obj1| *obj1);
//
//    //  while (!obj->memory.local_memory && obj) {
//    while obj.is_some() && obj.unwrap().memory().local_memory() == 0 {
//        // obj = hwloc_get_ancestor_obj_by_depth(topology, --depth, obj);
//        obj = obj.unwrap().parent();
//        depth -= 1;
//        //  }
//    }
//
//    //  assert(obj);
//    assert!(obj.is_some());
//    //  return depth;
//    Ok(depth as usize)
//}
//
////  static size_t alloc_from_node(cpu& this_cpu, hwloc_obj_t node, std::unordered_map<hwloc_obj_t, size_t>& used_mem, size_t alloc) {
//fn alloc_from_node(this_cpu: &mut Cpu,
//                   node: &TopologyObject,
//                   used_mem: &mut HashMap<*const TopologyObject, u64>,
//                   alloc: u64) -> usize {
//    //  auto taken = std::min(node->memory.local_memory - used_mem[node], alloc);
//    let pnode: *const TopologyObject = node;
//    let zero = 0;
//
//    //  auto taken = std::min(node->memory.local_memory - used_mem[node], alloc);
//    let used_mem_value = used_mem.entry(pnode).or_insert(0);
//    let taken = cmp::min(node.memory().local_memory() - *used_mem_value, alloc);
//
//    //  if (taken) {
//    if taken > 0 {
//        //  used_mem[node] += taken;
//        *used_mem_value += taken;
//
//        //  auto node_id = hwloc_bitmap_first(node->nodeset);
//        let node_id = node.nodeset().unwrap().first();
//
//        //  assert(node_id != -1);
//        assert!(node_id != -1);
//
//        //  this_cpu.mem.push_back({taken, unsigned(node_id)});
//        this_cpu.mem.push(Memory { bytes: taken as usize, nodeid: node_id as u32 })
//        //  }
//    }
//
//    //  return taken;
//    taken as usize
//    //  }
//}
//
////    distribute_objects(hwloc_topology_t& topology, size_t nobjs) : cpu_sets(nobjs), root(hwloc_get_root_obj(topology)) {
//fn distribute_objects(topology: &Topology, nobjs: usize) -> Option<Vec<CpuSet>> {
//
//    let root = topology.object_at_root();
//    //    #if HWLOC_API_VERSION >= 0x00010900
//    //    hwloc_distrib(topology, &root, 1, cpu_sets.data(), cpu_sets.size(), INT_MAX, 0);
//    //    #else
//    //    hwloc_distribute(topology, root, cpu_sets.data(), cpu_sets.size(), INT_MAX);
//    topology.distribute_objects(root, nobjs as u32)
//    //    #endif
//}
//
//fn get_pu_obj_for_cpu_id<'a>(topology: &'a Topology, cpu_id: CpuId) -> hwloc_error::Result<&'a TopologyObject> {
//    let pu_objs = try!(topology.objects_with_type(&ObjectType::PU));
//    let pu_obj_with_cpu_id = try!(pu_objs.into_iter()
//        .find(| &obj | obj.os_index() == cpu_id)
//        .ok_or("unable to find pu_obj with cpu_id"));
//    Ok(pu_obj_with_cpu_id)
//}
//
////static io_queue_topology
////allocate_io_queues(hwloc_topology_t& topology, configuration c, std::vector<cpu> cpus) {
//fn allocate_io_queues(topology: &Topology, configuration: &Configuration, cpus: &Vec<Cpu>) -> hwloc_error::Result<IoQueueTopology> {
//    //  unsigned num_io_queues = c.io_queues.value_or(cpus.size());
//    //  unsigned max_io_requests = c.max_io_requests.value_or(128 * num_io_queues);
//    let num_io_queues = configuration.get_io_queues().unwrap_or(cpus.len());
//    let max_io_requests = configuration.get_max_io_requests().unwrap_or(128 * num_io_queues);
//
//    //  unsigned depth = find_memory_depth(topology);
//    let depth = try!(find_memory_depth(&topology));
//
//    //  auto node_of_shard = [&topology, &cpus, &depth] (unsigned shard) {
//    let node_of_shard = |shard| -> hwloc_error::Result<i32> {
//        //  auto pu = hwloc_get_pu_obj_by_os_index(topology, cpus[shard].cpu_id);
//        let this_cpu = cpus.get(shard).unwrap();
//        let pu = try!(get_pu_obj_for_cpu_id(&topology, this_cpu.cpu_id));
//        //  auto node = hwloc_get_ancestor_obj_by_depth(topology, depth, pu);
//        let node = try!(pu.ancestor_by_depth(depth as u32).ok_or(""));
//        //  return hwloc_bitmap_first(node->nodeset);
//        let nodeset = try!(node.nodeset().ok_or(""));
//        Ok(nodeset.first())
//        //  };
//    };
//
//    //  // There are two things we are trying to achieve by populating a numa_nodes map.
//    //  //
//    //  // The first is to find out how many nodes we have in the system. We can't use
//    //  // hwloc for that, because at this point we are not longer talking about the physical system,
//    //  // but the actual booted seastar server instead. So if we have restricted the run to a subset
//    //  // of the available processors, counting topology nodes won't spur the same result.
//    //  //
//    //  // Secondly, we need to find out which processors live in each node. For a reason similar to the
//    //  // above, hwloc won't do us any good here. Later on, we will use this information to assign
//    //  // shards to coordinators that are node-local to themselves.
//    //  std::unordered_map<unsigned, std::set<unsigned>> numa_nodes;
//    let mut numa_nodes: HashMap<usize, HashSet<usize>> = HashMap::new();
//
//    //    for (auto shard: boost::irange(0, int(cpus.size()))) {
//    for (shard, this_cpu) in cpus.iter().enumerate() {
//        //  auto node_id = node_of_shard(shard);
//        let node_id = try!(node_of_shard(shard));
//        //
//        //    if (numa_nodes.count(node_id) == 0) {
//        //    numa_nodes.emplace(node_id, std::set<unsigned>());
//        //    }
//        //    numa_nodes.at(node_id).insert(shard);
//        //  }
//    }
//    //
//    //  io_queue_topology ret;
//    let ret = IoQueueTopology::default();
//    //    ret.shard_to_coordinator.resize(cpus.size());
//    //
//    //    // User may be playing with --smp option, but num_io_queues was independently
//    //    // determined by iotune, so adjust for any conflicts.
//    //    if (num_io_queues > cpus.size()) {
//    //    print("Warning: number of IO queues (%d) greater than logical cores (%d). Adjusting downwards.\n", num_io_queues, cpus.size());
//    //    num_io_queues = cpus.size();
//    //    }
//    //
//    //    auto find_shard = [&cpus] (unsigned cpu_id) {
//    //    auto idx = 0u;
//    //    for (auto& c: cpus) {
//    //    if (c.cpu_id == cpu_id) {
//    //    return idx;
//    //    }
//    //    idx++;
//    //    }
//    //    assert(0);
//    //    };
//    //
//    //    auto cpu_sets = distribute_objects(topology, num_io_queues);
//    //    // First step: distribute the IO queues given the information returned in cpu_sets.
//    //    // If there is one IO queue per processor, only this loop will be executed.
//    //    std::unordered_map<unsigned, std::vector<unsigned>> node_coordinators;
//    //    for (auto&& cs : cpu_sets()) {
//    //    auto io_coordinator = find_shard(hwloc_bitmap_first(cs));
//    //
//    //    ret.coordinators.emplace_back(io_queue{io_coordinator, std::max(max_io_requests / num_io_queues , 1u)});
//    //    // If a processor is a coordinator, it is also obviously a coordinator of itself
//    //    ret.shard_to_coordinator[io_coordinator] = io_coordinator;
//    //
//    //    auto node_id = node_of_shard(io_coordinator);
//    //    if (node_coordinators.count(node_id) == 0) {
//    //    node_coordinators.emplace(node_id, std::vector<unsigned>());
//    //    }
//    //    node_coordinators.at(node_id).push_back(io_coordinator);
//    //    numa_nodes[node_id].erase(io_coordinator);
//    //    }
//    //
//    //    // If there are more processors than coordinators, we will have to assign them to existing
//    //    // coordinators. We always do that within the same NUMA node.
//    //    for (auto& node: numa_nodes) {
//    //    auto cid_idx = 0;
//    //    for (auto& remaining_shard: node.second) {
//    //    auto idx = cid_idx++ % node_coordinators.at(node.first).size();
//    //    auto io_coordinator = node_coordinators.at(node.first)[idx];
//    //    ret.shard_to_coordinator[remaining_shard] = io_coordinator;
//    //    }
//    //    }
//
//    //  return ret;
//    Ok(ret)
//}
//
//pub fn allocate(c: Configuration) -> hwloc_error::Result<Resources> {
//
//    //  // Allocate the topology on stack
//    //  hwloc_topology_t topology;
//    //  // Init the struct
//    //  hwloc_topology_init(&topology);
//    //  // Defer deallocating the topology correctly
//    //  auto free_hwloc = defer([&] { hwloc_topology_destroy(topology); });
//    //  // Load the struct with the current topologuy
//    //  hwloc_topology_load(topology);
//    let topology = Topology::new();
//
//    //  if (c.cpu_set) {
//    if let Some(ref cpu_set) = c.cpu_set {
//        //  auto bm = hwloc_bitmap_alloc();
//        //  auto free_bm = defer([&] { hwloc_bitmap_free(bm); });
//        let mut bitmap = CpuSet::new();
//
//        //  for (auto idx : *c.cpu_set) {
//        //      hwloc_bitmap_set(bm, idx);
//        //  }
//        for &idx in cpu_set {
//            bitmap.set(idx);
//        }
//
//        //  auto r = hwloc_topology_restrict(topology, bm,
//        //                                   HWLOC_RESTRICT_FLAG_ADAPT_DISTANCES
//        //                                   | HWLOC_RESTRICT_FLAG_ADAPT_MISC
//        //                                   | HWLOC_RESTRICT_FLAG_ADAPT_IO);
//        //  if (r == -1) {
//        //      if (errno == ENOMEM) {
//        //          throw std::bad_alloc();
//        //      }
//        //  if (errno == EINVAL) {
//        //      throw std::runtime_error("bad cpuset");
//        //  }
//        //  abort();
//        //  }
//
//        //  }
//    }
//
//    //  auto machine_depth = hwloc_get_type_depth(topology, HWLOC_OBJ_MACHINE);
//    let machine_depth = try!(topology.depth_for_type(&ObjectType::Machine));
//
//    //  assert(hwloc_get_nbobjs_by_depth(topology, machine_depth) == 1);
//    let objects_at_depth = topology.objects_at_depth(machine_depth);
//    if objects_at_depth.len() != EXPECTED_SIZE_AT_DEPTH {
//        return Err(hwloc_error::ErrorKind::UnexpectedSizeAtDepth(machine_depth, objects_at_depth.len(), EXPECTED_SIZE_AT_DEPTH).into());
//    }
//
//    //  auto machine = hwloc_get_obj_by_depth(topology, machine_depth, 0);
//    let machine = objects_at_depth[0];
//
//    //  auto available_memory = machine->memory.total_memory;
//    let available_memory = machine.memory().total_memory() as usize;
//
//    // // hwloc doesn't account for kernel reserved memory, so set panic_factor = 2
//    // size_t mem = calculate_memory(c, available_memory, 2);
//    let mem: usize = try!(calculate_memory(&c, available_memory, DEFAULT_PANIC_FACTOR));
//
//    //  unsigned available_procs = hwloc_get_nbobjs_by_type(topology, HWLOC_OBJ_PU);
//    let available_procs: usize = try!(topology.objects_with_type(&ObjectType::PU)).len();
//
//    //  unsigned procs = c.cpus.value_or(available_procs);
//    let procs: usize = c.get_cpus().unwrap_or(available_procs);
//
//    //  if (procs > available_procs) {
//    if procs > available_procs {
//        //  throw std::runtime_error("insufficient processing units");
//        return Err(hwloc_error::ErrorKind::InsufficientProcessingUnits(procs, available_procs).into());
//    }
//
//    //  auto mem_per_proc = align_down<size_t>(mem / procs, 2 << 20);
//    let mem_per_proc: usize = align::align_down(mem / procs, (2 as usize).wrapping_shl(20));
//
//    //  resources ret;
//    let mut ret = Resources::default();
//
//    //  std::unordered_map<hwloc_obj_t, size_t> topo_used_mem;
//    let mut topo_used_mem: HashMap<*const TopologyObject, u64> = HashMap::new();
//
//    //  std::vector<std::pair<cpu, size_t>> remains;
//    let mut remains: Vec<(Cpu, usize)> = Vec::new();
//
//    //  size_t remain;
//    let mut remain: usize;
//
//    //  unsigned depth = find_memory_depth(topology);
//    let depth = try!(find_memory_depth(&topology));
//
//    let cpu_sets: Vec<CpuSet> = try!(distribute_objects(&topology, procs).ok_or("unable to distribute objects"));
//
//    // // Divide local memory to cpus
//    // for (auto&& cs : cpu_sets()) {
//    for cs in cpu_sets {
//        //  auto cpu_id = hwloc_bitmap_first(cs);
//        let cpu_id = cs.first();
//
//        //  assert(cpu_id != -1);
//        assert!(cpu_id != -1);
//
//        //  auto pu = hwloc_get_pu_obj_by_os_index(topology, cpu_id);
//        let pu = try!(get_pu_obj_for_cpu_id(&topology, cpu_id as u32));
//
//        //  auto node = hwloc_get_ancestor_obj_by_depth(topology, depth, pu);
//        let node = pu.ancestor_by_depth(depth as u32).unwrap();
//
//        //  cpu this_cpu;
//        //  this_cpu.cpu_id = cpu_id;
//        let mut this_cpu = Cpu { cpu_id: cpu_id as u32, mem: Vec::new() };
//
//        remain = mem_per_proc - alloc_from_node(&mut this_cpu, node, &mut topo_used_mem, mem_per_proc as u64);
//
//        //  remains.emplace_back(std::move(this_cpu), remain);
//        remains.push((this_cpu, remain));
//        //  }
//    }
//
//    //  // Divide the rest of the memory
//    //  for (auto&& r : remains) {
//    for (mut this_cpu, mut remain) in remains {
//        //  cpu this_cpu;
//        //  size_t remain;
//        //  std::tie(this_cpu, remain) = r;
//
//        //  auto pu = hwloc_get_pu_obj_by_os_index(topology, this_cpu.cpu_id);
//        let pu = try!(get_pu_obj_for_cpu_id(&topology, this_cpu.cpu_id));
//
//        //  auto node = hwloc_get_ancestor_obj_by_depth(topology, depth, pu);
//        let node = pu.ancestor_by_depth(depth as u32).unwrap();
//
//        //  auto obj = node;
//        let mut obj = Some(node);
//
//        //  while (remain) {
//        while remain > 0 {
//            //  remain -= alloc_from_node(this_cpu, obj, topo_used_mem, remain);
//            remain -= alloc_from_node(&mut this_cpu, node, &mut topo_used_mem, remain as u64);
//            //  do {
//            loop {
//                //  obj = hwloc_get_next_obj_by_depth(topology, depth, obj);
//                //  } while (!obj);
//            }
//            //            if (obj == node)
//            //                break;
//            //  }
//        }
//        //  assert(!remain);
//        assert!(remain == 0);
//        //  ret.cpus.push_back(std::move(this_cpu));
//        ret.cpus.push(this_cpu);
//        //  }
//    }
//
//    //  ret.io_queues = allocate_io_queues(topology, c, ret.cpus);
//    let io_queues = try!(allocate_io_queues(&topology, &c, ret.get_cpus()));
//
//    //  return ret;
//    Ok(ret)
//}
