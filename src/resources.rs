use std::vec::Vec;
use std::collections::HashSet;
use error::resources_error;
use resource::nix;

pub type CpuId = u32;

#[derive(Default, Debug, Builder, Clone)]
pub struct Cpu {
    cpu_id: CpuId,
    mem: Vec<Memory>,
}

#[derive(Default, Debug, Builder, Clone)]
pub struct Memory {
    bytes: usize,
    nodeid: u32,
}

#[derive(Default, Debug, Builder, Clone)]
pub struct IoQueue {
    id: usize,
    capacity: usize,
}

#[derive(Default, Debug, Builder)]
pub struct Configuration {
    total_memory: Option<usize>,
    reserve_memory: Option<usize>, // if total_memory not specified
    cpus: Option<usize>,
    cpu_set: Option<HashSet<CpuId>>,
    max_io_requests: Option<usize>,
    io_queues: Option<usize>,
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

    pub fn get_cpu_set(&self) -> Option<&HashSet<CpuId>> {
        return self.cpu_set.as_ref();
    }

    pub fn get_max_io_requests(&self) -> Option<usize> {
        return self.max_io_requests;
    }

    pub fn get_io_queues(&self) -> Option<usize> {
        return self.io_queues;
    }
}

// Since this is static information, we will keep a copy at each CPU.
// This will allow us to easily find who is the IO coordinator for a given
// node without a trip to a remote CPU.
#[derive(Default, Debug, Builder, Clone)]
pub struct IoQueueTopology {
    shard_to_coordinator: Vec<usize>,
    coordinators: Vec<IoQueue>,
}

impl IoQueueTopology {
    pub fn get_shard_to_coordinator_mut(&mut self) -> &mut Vec<usize> {
        &mut self.shard_to_coordinator
    }

    pub fn get_coordinators_mut(&mut self) -> &mut Vec<IoQueue> {
        &mut self.coordinators
    }
}

#[derive(Default, Debug, Builder, Clone)]
pub struct Resources {
    cpus: Vec<Cpu>,
    io_queues: IoQueueTopology,
}

impl Resources {
    pub fn get_cpus(&self) -> &Vec<Cpu> {
        &self.cpus
    }

    pub fn get_cpus_mut(&mut self) -> &mut Vec<Cpu> {
        &mut self.cpus
    }
}

pub fn allocate(c: Configuration) -> resources_error::Result<Resources> {
    nix::allocate(c)
}
