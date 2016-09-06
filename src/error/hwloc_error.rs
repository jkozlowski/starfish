extern crate hwloc;

use hwloc::{TypeDepthError};

error_chain! {
  types {
    Error, ErrorKind, ChainErr, Result;
  }

  links {
  }

  foreign_links {
    TypeDepthError, Hwloc;
  }

  errors {
    UnexpectedSizeAtDepth(machine_depth: u32, actual_size: usize, expected_size: usize) {
      description("unexpected size at machine depth")
      display("unexpected size ({}) at machine depth ({}): expected {}",
              actual_size, machine_depth, expected_size)
    }
    InsufficientPhysicalMemory(requested_memory: usize, available_memory: usize) {
      description("insufficient physical memory")
      display("insufficient physical memory: requested_memory({}) > available_memory({})",
              requested_memory, available_memory)
    }
  }
}