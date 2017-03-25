extern crate tokio_smp;

use tokio_smp::resources;
use tokio_smp::resources::Configuration;
use tokio_smp::resources::ConfigurationBuilder;

/// Example which binds an arbitrary process (in this example this very same one) to
/// the last core.
fn main() {

    let c = ConfigurationBuilder::default()
                          //.total_memory(Some(17179869186))
                          .cpu_set(Some(vec!(0,1).into_iter().collect()))
                          .cpus(Some(2))
                          .build()
                          .unwrap();

    match resources::allocate(c) {
        Ok(i) => println!("Result: {:?}", i),
        Err(e) => println!("Could not allocate: {:?}", e),
    }

    println!("Thank you!");
}
