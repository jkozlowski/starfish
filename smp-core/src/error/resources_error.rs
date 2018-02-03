error_chain! {
  types {
    Error, ErrorKind, ChainErr, Result;
  }

  links {
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
    InsufficientProcessingUnits(requested_pus: usize, available_pus: usize) {
      description("insufficient processing units")
      display("insufficient processing units: requested ({}) > available({})",
              requested_pus, available_pus)
    }
  }
}
