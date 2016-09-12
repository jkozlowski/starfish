extern crate libc;

pub mod hwloc;
pub mod hwloc_error;
pub mod nix;

use error::resources_error;
use resources::{Configuration};
use std::cmp::{Ordering};

const DEFAULT_PANIC_FACTOR: f32 = 1.0;

fn calculate_memory_default_panic_factor(c: &Configuration,
                                         available_memory: usize)
                                         -> resources_error::Result<usize> {
    calculate_memory(c, available_memory, DEFAULT_PANIC_FACTOR)
}

fn calculate_memory(c: &Configuration,
                    mut available_memory: usize,
                    panic_factor: f32)
                    -> resources_error::Result<usize> {
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
        return Err(resources_error::ErrorKind::InsufficientPhysicalMemory(mem, available_memory).into());
    } else {
        //  return mem;
        return Ok(mem);
    }
}

fn memory_to_reserve(useable_memory: f32) -> f32 {
    let min_memory: f32 = 1u32.wrapping_shl(30) as f32;
    match min_memory.partial_cmp(&useable_memory).unwrap_or(Ordering::Equal) {
        Ordering::Equal => min_memory,
        Ordering::Less => useable_memory,
        Ordering::Greater => min_memory
    }
}